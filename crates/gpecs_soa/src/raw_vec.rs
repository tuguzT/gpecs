use alloc::alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error, realloc};
use core::{
    alloc::{Layout, LayoutError},
    cmp,
    fmt::{self, Display},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::ptr::{align_of_buffer, min_size_of, ptrs, to_len, to_len_in_bytes, BufferAlign};

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

// impl core::error::Error for TryReserveError {}

#[inline(never)]
const fn capacity_overflow() -> ! {
    panic!("capacity overflow");
}

enum AllocInit {
    /// The contents of the new memory are uninitialized.
    Uninitialized,
    /// The new memory is guaranteed to be zeroed.
    #[allow(dead_code)]
    Zeroed,
}

pub struct RawSoaVec<T, U, V> {
    ptr: NonNull<BufferAlign<T, U, V>>,
    buffer_capacity: usize,
    phantom: PhantomData<(T, U, V)>,
}

impl<T, U, V> RawSoaVec<T, U, V> {
    // Tiny Vecs are dumb. Skip to:
    // - 8 if the element size is 1, because any heap allocators is likely
    //   to round up a request of less than 8 bytes to at least 8 bytes.
    // - 4 if elements are moderate-sized (<= 1 KiB).
    // - 1 otherwise, to avoid wasting too much space for very short Vecs.
    pub(crate) const MIN_NON_ZERO_CAP: usize = if min_size_of::<T, U, V>() == 1 {
        8
    } else if min_size_of::<T, U, V>() <= 1024 {
        4
    } else {
        1
    };

    pub const LAYOUT_ALIGN: usize = align_of_buffer::<T, U, V>();

    #[must_use]
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            buffer_capacity: 0,
            phantom: PhantomData,
        }
    }

    fn try_allocate_in(capacity: usize, init: AllocInit) -> Result<Self, TryReserveError> {
        if capacity == 0 || min_size_of::<T, U, V>() == 0 {
            return Ok(Self::new());
        }

        let size = to_len_in_bytes::<T, U, V>(capacity);
        let layout = match Layout::from_size_align(size, Self::LAYOUT_ALIGN) {
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

        Ok(Self {
            ptr: ptr.cast(),
            buffer_capacity: layout.size(),
            phantom: PhantomData,
        })
    }

    pub fn with_capacity(capacity: usize) -> Self {
        match Self::try_with_capacity(capacity) {
            Ok(me) => me,
            Err(err) => handle_error(err),
        }
    }

    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        Self::try_allocate_in(capacity, AllocInit::Uninitialized)
    }

    #[allow(dead_code)]
    pub fn with_capacity_zeroed(capacity: usize) -> Self {
        match Self::try_with_capacity_zeroed(capacity) {
            Ok(me) => me,
            Err(err) => handle_error(err),
        }
    }

    #[allow(dead_code)]
    pub fn try_with_capacity_zeroed(capacity: usize) -> Result<Self, TryReserveError> {
        Self::try_allocate_in(capacity, AllocInit::Zeroed)
    }

    pub const unsafe fn from_raw_parts(ptr: *mut u8, capacity: usize) -> Self {
        unsafe {
            let ptr = NonNull::new_unchecked(ptr);
            Self::from_nonnull(ptr, capacity)
        }
    }

    pub const unsafe fn from_nonnull(ptr: NonNull<u8>, capacity: usize) -> Self {
        let buffer_capacity = if min_size_of::<T, U, V>() == 0 {
            0
        } else {
            to_len_in_bytes::<T, U, V>(capacity)
        };

        Self {
            ptr: ptr.cast(),
            buffer_capacity,
            phantom: PhantomData,
        }
    }

    pub const fn ptr(&self) -> *mut u8 {
        self.non_null().as_ptr()
    }

    pub const fn non_null(&self) -> NonNull<u8> {
        self.ptr.cast()
    }

    #[allow(dead_code)]
    pub const fn ptr_without_header(&self) -> Option<*mut u8> {
        if self.buffer_capacity == 0 {
            return None;
        }

        Some(unsafe { self.ptr().byte_add(size_of::<usize>()) })
    }

    #[allow(dead_code)]
    pub const fn non_null_without_header(&self) -> Option<NonNull<u8>> {
        match self.ptr_without_header() {
            Some(ptr) => Some(unsafe { NonNull::new_unchecked(ptr) }),
            None => None,
        }
    }

    pub fn ptrs(&self) -> (*mut T, *mut U, *mut V) {
        let ptr = self.ptr();
        let len = self.capacity();

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[allow(dead_code)]
    pub fn non_nulls(&self) -> (NonNull<T>, NonNull<U>, NonNull<V>) {
        let (t_ptr, u_ptr, v_ptr) = self.ptrs();
        unsafe {
            (
                NonNull::new_unchecked(t_ptr),
                NonNull::new_unchecked(u_ptr),
                NonNull::new_unchecked(v_ptr),
            )
        }
    }

    pub const fn capacity(&self) -> usize {
        if min_size_of::<T, U, V>() == 0 {
            usize::MAX
        } else {
            to_len::<T, U, V>(self.buffer_capacity)
        }
    }

    pub const fn buffer_capacity(&self) -> usize {
        self.buffer_capacity
    }

    const fn current_memory(&self) -> Option<(NonNull<u8>, Layout)> {
        if min_size_of::<T, U, V>() == 0 || self.buffer_capacity == 0 {
            return None;
        }

        // We could use Layout::from_size_align here which ensures the absence of isize and usize overflows
        // and could hypothetically handle differences between stride and size, but this memory
        // has already been allocated so we know it can't overflow and currently Rust does not
        // support such types. So we can do better by skipping some checks and avoid an unwrap.
        unsafe {
            let size = self.buffer_capacity;
            let layout = Layout::from_size_align_unchecked(size, Self::LAYOUT_ALIGN);
            Some((self.ptr.cast(), layout))
        }
    }

    pub fn reserve(&mut self, len: usize, additional: usize) {
        // Callers expect this function to be very cheap when there is already sufficient capacity.
        // Therefore, we move all the resizing and error-handling logic from grow_amortized and
        // handle_reserve behind a call, while making sure that this function is likely to be
        // inlined as just a comparison and a call if the comparison fails.
        #[cold]
        fn do_reserve_and_handle<T, U, V>(
            slf: &mut RawSoaVec<T, U, V>,
            len: usize,
            additional: usize,
        ) {
            if let Err(err) = slf.grow_amortized(len, additional) {
                handle_error(err);
            }
        }

        if self.needs_to_grow(len, additional) {
            do_reserve_and_handle(self, len, additional);
        }
    }

    pub fn grow_one(&mut self) {
        if let Err(err) = self.grow_amortized(self.capacity(), 1) {
            handle_error(err);
        }
    }

    pub fn try_reserve(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_amortized(len, additional)?;
        }
        Ok(())
    }

    pub fn reserve_exact(&mut self, len: usize, additional: usize) {
        if let Err(err) = self.try_reserve_exact(len, additional) {
            handle_error(err);
        }
    }

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

    pub fn shrink_to_fit(&mut self, cap: usize) {
        if let Err(err) = self.shrink(cap) {
            handle_error(err);
        }
    }

    pub fn needs_to_grow(&self, len: usize, additional: usize) -> bool {
        let new_buffer_capacity = to_len_in_bytes::<T, U, V>(len + additional);
        new_buffer_capacity > self.buffer_capacity
    }

    unsafe fn set_ptr_and_cap(&mut self, ptr: NonNull<u8>, cap: usize) {
        self.ptr = ptr.cast();
        self.buffer_capacity = to_len_in_bytes::<T, U, V>(cap);
    }

    fn grow_amortized(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        debug_assert!(additional > 0);

        if min_size_of::<T, U, V>() == 0 {
            return Err(CapacityOverflow.into());
        }

        let required_cap = len.checked_add(additional).ok_or(CapacityOverflow)?;

        let cap = cmp::max(self.buffer_capacity * 2, required_cap);
        let cap = cmp::max(Self::MIN_NON_ZERO_CAP, cap);

        let layout_size = to_len_in_bytes::<T, U, V>(cap);
        let new_layout = Layout::from_size_align(layout_size, Self::LAYOUT_ALIGN);

        let ptr = finish_grow(new_layout, self.current_memory())?;
        unsafe {
            self.set_ptr_and_cap(ptr, cap);
        }
        Ok(())
    }

    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        if min_size_of::<T, U, V>() == 0 {
            return Err(CapacityOverflow.into());
        }

        let cap = len.checked_add(additional).ok_or(CapacityOverflow)?;

        let layout_size = to_len_in_bytes::<T, U, V>(cap);
        let new_layout = Layout::from_size_align(layout_size, Self::LAYOUT_ALIGN);

        let ptr = finish_grow(new_layout, self.current_memory())?;
        unsafe {
            self.set_ptr_and_cap(ptr, cap);
        }
        Ok(())
    }

    fn shrink(&mut self, cap: usize) -> Result<(), TryReserveError> {
        assert!(
            cap <= self.capacity(),
            "tried to shrink to a larger capacity",
        );

        let (ptr, old_layout) = match self.current_memory() {
            Some(mem) => mem,
            None => return Ok(()),
        };

        if cap == 0 {
            unsafe {
                dealloc(ptr.as_ptr(), old_layout);
                self.set_ptr_and_cap(NonNull::dangling(), 0);
            }
            return Ok(());
        }

        let ptr = unsafe {
            let layout_size = to_len_in_bytes::<T, U, V>(cap);
            let new_layout = Layout::from_size_align_unchecked(layout_size, Self::LAYOUT_ALIGN);

            let ptr = realloc(ptr.as_ptr(), old_layout, new_layout.size());
            match NonNull::new(ptr) {
                Some(ptr) => ptr,
                #[rustfmt::skip]
                None => return Err(AllocError { layout: new_layout, non_exhaustive: () }.into()),
            }
        };
        unsafe {
            self.set_ptr_and_cap(ptr, cap);
        }
        Ok(())
    }
}

impl<T, U, V> Drop for RawSoaVec<T, U, V> {
    fn drop(&mut self) {
        if let Some((ptr, layout)) = self.current_memory() {
            unsafe { dealloc(ptr.as_ptr(), layout) }
        }
    }
}

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
