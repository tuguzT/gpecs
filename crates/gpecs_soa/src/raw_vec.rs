use alloc::{
    alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error, realloc},
    boxed::Box,
};
use core::{
    alloc::{Layout, LayoutError},
    cmp,
    error::Error,
    fmt::{self, Display},
    mem::ManuallyDrop,
    ptr::NonNull,
};

use crate::{
    ptr::{
        actual_capacity, buffer_layout, is_zst, ptrs, slice_from_raw_parts_mut, to_capacity,
        BufferData,
    },
    slice::SoaSlice,
    traits::Soa,
};

use self::TryReserveErrorKind::*;

/// The error type for `try_reserve` methods.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TryReserveError {
    kind: TryReserveErrorKind,
}

impl TryReserveError {
    /// Details about the allocation that caused the error.
    #[inline]
    #[must_use]
    pub fn kind(&self) -> TryReserveErrorKind {
        self.kind.clone()
    }
}

/// Details of the allocation that caused a [`TryReserveError`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TryReserveErrorKind {
    /// Error due to the computed capacity exceeding the collection's maximum
    /// (usually `isize::MAX` bytes).
    CapacityOverflow,

    /// The memory allocator returned an error.
    AllocError {
        /// The layout of allocation request that failed.
        layout: Layout,

        #[doc(hidden)]
        non_exhaustive: (),
    },
}

impl From<TryReserveErrorKind> for TryReserveError {
    #[inline]
    fn from(kind: TryReserveErrorKind) -> Self {
        Self { kind }
    }
}

impl From<LayoutError> for TryReserveErrorKind {
    /// Always evaluates to [`TryReserveErrorKind::CapacityOverflow`].
    #[inline]
    fn from(_: LayoutError) -> Self {
        CapacityOverflow
    }
}

impl Display for TryReserveError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("memory allocation failed")?;

        let reason = match self.kind {
            CapacityOverflow => " because the computed capacity exceeded the collection's maximum",
            AllocError { .. } => " because the memory allocator returned an error",
        };
        fmt.write_str(reason)
    }
}

impl Error for TryReserveError {}

#[inline(never)]
const fn capacity_overflow() -> ! {
    panic!("capacity overflow");
}

enum AllocInit {
    /// The contents of the new memory are uninitialized.
    Uninitialized,
    /// The new memory is guaranteed to be zeroed.
    Zeroed,
}

pub struct RawSoaVec<T>
where
    T: Soa,
{
    ptr: NonNull<BufferData<T>>,
    capacity_in_bytes: usize,
    #[cfg(feature = "cache-ptrs")]
    ptrs: T::NonNullPtrs,
}

impl<T> RawSoaVec<T>
where
    T: Soa,
{
    // Tiny Vecs are dumb. Skip to:
    // - 8 if the element size is 1, because any heap allocators is likely
    //   to round up a request of less than 8 bytes to at least 8 bytes.
    // - 4 if elements are moderate-sized (<= 1 KiB).
    // - 1 otherwise, to avoid wasting too much space for very short Vecs.
    #[inline]
    pub fn min_non_zero_cap() -> usize {
        if T::packed_size_of() == 1 {
            8
        } else if T::packed_size_of() <= 1024 {
            4
        } else {
            1
        }
    }

    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity_in_bytes: 0,
            #[cfg(feature = "cache-ptrs")]
            ptrs: unsafe { T::ptrs_to_nonnull(T::ptrs_dangling()) },
        }
    }

    fn try_allocate_in(capacity: usize, init: AllocInit) -> Result<Self, TryReserveError> {
        if is_zst::<T>() || capacity == 0 {
            return Ok(Self::new());
        }

        let capacity = actual_capacity::<T>(capacity);
        let layout = match buffer_layout::<T>(capacity) {
            Ok(layout) => layout,
            Err(_) => return Err(CapacityOverflow.into()),
        };
        alloc_guard(layout.size())?;

        let ptr = match init {
            AllocInit::Uninitialized => unsafe { alloc(layout) },
            AllocInit::Zeroed => unsafe { alloc_zeroed(layout) },
        };
        let ptr = match NonNull::new(ptr) {
            Some(ptr) => ptr,
            #[rustfmt::skip]
            None => return Err(AllocError { layout, non_exhaustive: () }.into()),
        };

        let ptr = ptr.cast();
        Ok(Self {
            ptr,
            capacity_in_bytes: layout.size(),
            #[cfg(feature = "cache-ptrs")]
            ptrs: unsafe {
                let ptrs = ptrs::<T>(ptr.as_ptr(), capacity).unwrap_unchecked();
                T::ptrs_to_nonnull(ptrs)
            },
        })
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        match Self::try_with_capacity(capacity) {
            Ok(me) => me,
            Err(err) => handle_error(err),
        }
    }

    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        Self::try_allocate_in(capacity, AllocInit::Uninitialized)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn with_capacity_zeroed(capacity: usize) -> Self {
        match Self::try_with_capacity_zeroed(capacity) {
            Ok(me) => me,
            Err(err) => handle_error(err),
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn try_with_capacity_zeroed(capacity: usize) -> Result<Self, TryReserveError> {
        Self::try_allocate_in(capacity, AllocInit::Zeroed)
    }

    #[inline]
    pub unsafe fn into_box(self, len: usize) -> Box<SoaSlice<T>> {
        debug_assert!(
            len <= self.capacity(),
            "`len` must be smaller than or equal to `self.capacity()`"
        );

        let me = ManuallyDrop::new(self);
        unsafe {
            let slice = slice_from_raw_parts_mut(me.ptr(), len, me.capacity());
            Box::from_raw(slice)
        }
    }

    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut BufferData<T>, capacity: usize) -> Self {
        unsafe {
            let ptr = NonNull::new_unchecked(ptr);
            Self::from_nonnull(ptr, capacity)
        }
    }

    #[inline]
    pub unsafe fn from_nonnull(ptr: NonNull<BufferData<T>>, capacity: usize) -> Self {
        let capacity_in_bytes = buffer_layout::<T>(capacity)
            .expect("layout size should not exceed `isize::MAX`")
            .size();

        Self {
            ptr,
            capacity_in_bytes,
            #[cfg(feature = "cache-ptrs")]
            ptrs: unsafe {
                let ptrs = ptrs::<T>(ptr.as_ptr(), capacity).unwrap_unchecked();
                T::ptrs_to_nonnull(ptrs)
            },
        }
    }

    #[inline]
    pub fn ptr(&self) -> *mut BufferData<T> {
        self.non_null().as_ptr()
    }

    #[inline]
    pub fn non_null(&self) -> NonNull<BufferData<T>> {
        self.ptr
    }

    #[inline]
    #[cfg(not(feature = "cache-ptrs"))]
    pub fn ptrs(&self) -> T::MutPtrs {
        let ptr = self.ptr();
        let capacity = self.capacity();

        unsafe { ptrs::<T>(ptr, capacity).unwrap_unchecked() }
    }

    #[inline]
    #[cfg(feature = "cache-ptrs")]
    pub fn ptrs(&self) -> T::MutPtrs {
        let ptrs = self.ptrs.clone();
        T::nonnull_to_ptrs(ptrs)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn non_nulls(&self) -> T::NonNullPtrs {
        let ptrs = self.ptrs();
        unsafe { T::ptrs_to_nonnull(ptrs) }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        if is_zst::<T>() {
            return usize::MAX;
        }
        to_capacity::<T>(self.capacity_in_bytes)
    }

    #[inline(always)]
    pub fn capacity_in_bytes(&self) -> usize {
        self.capacity_in_bytes
    }

    #[inline]
    const fn current_memory(&self) -> Option<(NonNull<u8>, Layout)> {
        if self.capacity_in_bytes == 0 {
            return None;
        }

        // We could use Layout::from_size_align here which ensures the absence of isize and usize overflows
        // and could hypothetically handle differences between stride and size, but this memory
        // has already been allocated so we know it can't overflow and currently Rust does not
        // support such types. So we can do better by skipping some checks and avoid an unwrap.
        unsafe {
            let size = self.capacity_in_bytes;
            let layout = Layout::from_size_align_unchecked(size, align_of::<BufferData<T>>());
            Some((self.ptr.cast(), layout))
        }
    }

    #[inline]
    pub fn reserve(&mut self, len: usize, additional: usize) {
        // Callers expect this function to be very cheap when there is already sufficient capacity.
        // Therefore, we move all the resizing and error-handling logic from grow_amortized and
        // handle_reserve behind a call, while making sure that this function is likely to be
        // inlined as just a comparison and a call if the comparison fails.
        #[cold]
        fn do_reserve_and_handle<T>(this: &mut RawSoaVec<T>, len: usize, additional: usize)
        where
            T: Soa,
        {
            if let Err(err) = this.grow_amortized(len, additional) {
                handle_error(err);
            }
        }

        if self.needs_to_grow(len, additional) {
            do_reserve_and_handle(self, len, additional);
        }
    }

    #[inline]
    pub fn grow_one(&mut self) {
        if let Err(err) = self.grow_amortized(self.capacity(), 1) {
            handle_error(err);
        }
    }

    #[inline]
    pub fn try_reserve(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_amortized(len, additional)?;
        }
        Ok(())
    }

    #[inline]
    pub fn reserve_exact(&mut self, len: usize, additional: usize) {
        if let Err(err) = self.try_reserve_exact(len, additional) {
            handle_error(err);
        }
    }

    #[inline]
    pub fn try_reserve_exact(
        &mut self,
        len: usize,
        additional: usize,
    ) -> Result<(), TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_exact(len, additional)?;
        }
        Ok(())
    }

    #[inline]
    pub fn shrink_to_fit(&mut self, capacity: usize) {
        if let Err(err) = self.shrink(capacity) {
            handle_error(err);
        }
    }

    #[inline]
    pub fn needs_to_grow(&self, len: usize, additional: usize) -> bool {
        let new_capacity_in_bytes = buffer_layout::<T>(len + additional)
            .expect("layout size should not exceed `isize::MAX`")
            .size();
        new_capacity_in_bytes > self.capacity_in_bytes
    }

    #[inline]
    unsafe fn set_ptr_and_capacity(&mut self, ptr: NonNull<BufferData<T>>, capacity: usize) {
        self.ptr = ptr;
        self.capacity_in_bytes = buffer_layout::<T>(capacity)
            .expect("layout size should not exceed `isize::MAX`")
            .size();

        #[cfg(feature = "cache-ptrs")]
        {
            self.ptrs = unsafe {
                let ptrs = if ptr == NonNull::dangling() {
                    T::ptrs_dangling()
                } else {
                    ptrs::<T>(ptr.as_ptr(), capacity).unwrap_unchecked()
                };
                T::ptrs_to_nonnull(ptrs)
            };
        }
    }

    fn grow_amortized(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        debug_assert!(additional > 0);

        if is_zst::<T>() {
            return Err(CapacityOverflow.into());
        }

        let required_capacity = len.checked_add(additional).ok_or(CapacityOverflow)?;

        let capacity = cmp::max(self.capacity() * 2, required_capacity);
        let capacity = cmp::max(Self::min_non_zero_cap(), capacity);
        let capacity = actual_capacity::<T>(capacity);
        let new_layout = buffer_layout::<T>(capacity);

        let ptr = finish_grow(new_layout, self.current_memory())?;
        unsafe {
            self.set_ptr_and_capacity(ptr.cast(), capacity);
        }
        Ok(())
    }

    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        if is_zst::<T>() {
            return Err(CapacityOverflow.into());
        }

        let capacity = len.checked_add(additional).ok_or(CapacityOverflow)?;
        let capacity = actual_capacity::<T>(capacity);
        let new_layout = buffer_layout::<T>(capacity);

        let ptr = finish_grow(new_layout, self.current_memory())?;
        unsafe {
            self.set_ptr_and_capacity(ptr.cast(), capacity);
        }
        Ok(())
    }

    fn shrink(&mut self, capacity: usize) -> Result<(), TryReserveError> {
        assert!(
            capacity <= self.capacity(),
            "tried to shrink to a larger capacity",
        );

        let (ptr, old_layout) = match self.current_memory() {
            Some(mem) => mem,
            None => return Ok(()),
        };

        if capacity == 0 {
            unsafe {
                dealloc(ptr.as_ptr(), old_layout);
                self.set_ptr_and_capacity(NonNull::dangling(), 0);
            }
            return Ok(());
        }

        let ptr = unsafe {
            let new_layout = match buffer_layout::<T>(capacity) {
                Ok(layout) => layout,
                Err(_) => return Err(CapacityOverflow.into()),
            };
            if new_layout.size() == 0 {
                return Ok(());
            }

            let ptr = realloc(ptr.as_ptr(), old_layout, new_layout.size());
            match NonNull::new(ptr) {
                Some(ptr) => ptr,
                #[rustfmt::skip]
                None => return Err(AllocError { layout: new_layout, non_exhaustive: () }.into()),
            }
        };
        unsafe {
            self.set_ptr_and_capacity(ptr.cast(), capacity);
        }
        Ok(())
    }
}

impl<T> Drop for RawSoaVec<T>
where
    T: Soa,
{
    fn drop(&mut self) {
        if let Some((ptr, layout)) = self.current_memory() {
            unsafe { dealloc(ptr.as_ptr(), layout) }
        }
    }
}

unsafe impl<T> Send for RawSoaVec<T> where T: Soa + Send {}
unsafe impl<T> Sync for RawSoaVec<T> where T: Soa + Sync {}

#[inline(never)]
fn finish_grow(
    new_layout: Result<Layout, LayoutError>,
    current_memory: Option<(NonNull<u8>, Layout)>,
) -> Result<NonNull<u8>, TryReserveError> {
    // Check for the error here to minimize the size of `RawVec::grow_*`.
    let new_layout = new_layout.map_err(|_| CapacityOverflow)?;

    alloc_guard(new_layout.size())?;

    let ptr = if let Some((ptr, old_layout)) = current_memory {
        debug_assert_eq!(old_layout.align(), new_layout.align());
        unsafe { realloc(ptr.as_ptr(), old_layout, new_layout.size()) }
    } else {
        unsafe { alloc(new_layout) }
    };

    match NonNull::new(ptr) {
        Some(ptr) => Ok(ptr),
        #[rustfmt::skip]
        None => Err(AllocError { layout: new_layout, non_exhaustive: () }.into()),
    }
}

#[cold]
fn handle_error(error: TryReserveError) -> ! {
    match error.kind() {
        CapacityOverflow => capacity_overflow(),
        AllocError { layout, .. } => handle_alloc_error(layout),
    }
}

#[inline(always)]
fn alloc_guard(alloc_size: usize) -> Result<(), TryReserveError> {
    if usize::BITS < 64 && alloc_size > isize::MAX as usize {
        Err(CapacityOverflow.into())
    } else {
        Ok(())
    }
}
