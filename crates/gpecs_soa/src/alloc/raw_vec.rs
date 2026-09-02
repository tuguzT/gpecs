use core::{
    alloc::Layout,
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};
use core_alloc::{
    alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error, realloc},
    boxed::Box,
};

use crate::{
    alloc::error::{
        TryReserveError,
        TryReserveErrorKind::{AllocError, CapacityOverflow},
        alloc_error,
    },
    buffer::{
        BufferDropCheck, BufferPrefix, buffer_align, buffer_layout, buffer_layout_capacity,
        capacity_from, layout_is_dangling, ptr_to_buffer_context_mut, ptr_to_buffer_prefix_mut,
        ptrs_from_buffer_mut,
    },
    ptr::slice_from_raw_parts_mut,
    slice::SoaSlice,
    traits::{AllocSoa, AllocSoaContext, AllocSoaTrusted, MutPtrs},
};

#[derive(Debug, Clone, Copy)]
enum AllocInit {
    /// The contents of the new memory are uninitialized.
    Uninitialized,
    /// The new memory is guaranteed to be zeroed.
    Zeroed,
}

pub struct RawSoaVec<T>
where
    T: AllocSoa + ?Sized,
{
    ptr: NonNull<u8>,
    capacity: usize,
    _marker: BufferDropCheck<T>,
}

impl<T> RawSoaVec<T>
where
    T: AllocSoa + ?Sized,
{
    // Tiny Vecs are dumb. Skip to:
    // - 8 if the element size is 1, because any heap allocators is likely
    //   to round up a request of less than 8 bytes to at least 8 bytes.
    // - 4 if elements are moderate-sized (<= 1 KiB).
    // - 1 otherwise, to avoid wasting too much space for very short Vecs.
    #[inline]
    pub fn min_non_zero_cap(context: &T::Context) -> usize {
        const SIZE: usize = 4096; // 4 KiB

        let align = buffer_align::<T>(context);
        let Ok(buffer_layout) = Layout::from_size_align(SIZE, align) else {
            return 1;
        };

        match capacity_from::<T>(context, buffer_layout) {
            SIZE.. => 8,
            4.. => 4,
            _ => 1,
        }
    }

    fn try_allocate_in(
        context: T::Context,
        capacity: usize,
        init: AllocInit,
    ) -> Result<Self, TryReserveError> {
        let Ok((layout, capacity)) = buffer_layout_capacity::<T>(&context, capacity) else {
            return Err(CapacityOverflow.into());
        };
        if layout_is_dangling(layout) {
            let ptr = layout.dangling_ptr();
            let this = unsafe { Self::from_nonnull(ptr, capacity) };
            return Ok(this);
        }

        let ptr = match init {
            AllocInit::Uninitialized => unsafe { alloc(layout) },
            AllocInit::Zeroed => unsafe { alloc_zeroed(layout) },
        };
        let Some(ptr) = NonNull::new(ptr) else {
            return Err(alloc_error(layout).into());
        };

        let mut me = unsafe { Self::from_nonnull(ptr, capacity) };
        unsafe { ptr::write(me.ptr_to_context(), context) }
        me.set_capacity_in_buffer();

        Ok(me)
    }

    #[inline]
    pub fn ptr_to_context(&self) -> *mut T::Context {
        let buffer = self.as_ptr();
        unsafe { ptr_to_buffer_context_mut::<T>(buffer) }
    }

    #[inline]
    pub fn ptr_to_prefix(&self) -> Option<*mut BufferPrefix<T>> {
        let Self { ptr, capacity, .. } = *self;
        let context = self.context();
        let buffer = ptr.as_ptr();

        unsafe { ptr_to_buffer_prefix_mut::<T>(context, capacity, buffer).unwrap_unchecked() }
    }

    #[inline]
    fn set_capacity_in_buffer(&mut self) {
        let Some(prefix) = self.ptr_to_prefix() else {
            return;
        };

        let capacity = self.capacity();
        let ptr_to_capacity = unsafe { &raw mut (*prefix).capacity };
        unsafe { ptr::write(ptr_to_capacity, capacity) }
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(context: T::Context, capacity: usize) -> Self {
        match Self::try_with_capacity(context, capacity) {
            Ok(me) => me,
            Err(err) => handle_error(err),
        }
    }

    #[inline]
    pub fn try_with_capacity(
        context: T::Context,
        capacity: usize,
    ) -> Result<Self, TryReserveError> {
        Self::try_allocate_in(context, capacity, AllocInit::Uninitialized)
    }

    #[inline]
    #[must_use]
    #[expect(dead_code)]
    pub fn with_capacity_zeroed(context: T::Context, capacity: usize) -> Self {
        match Self::try_with_capacity_zeroed(context, capacity) {
            Ok(me) => me,
            Err(err) => handle_error(err),
        }
    }

    #[inline]
    pub fn try_with_capacity_zeroed(
        context: T::Context,
        capacity: usize,
    ) -> Result<Self, TryReserveError> {
        Self::try_allocate_in(context, capacity, AllocInit::Zeroed)
    }

    #[inline]
    unsafe fn deallocate(&mut self) -> T::Context {
        let context = self.ptr_to_context();
        // move context onto the stack to safely return it after buffer deallocation
        let context = unsafe { ptr::read(context) };

        if let Some((ptr, layout)) = self.current_memory(&context) {
            unsafe { dealloc(ptr.as_ptr(), layout) }
        }
        context
    }

    #[inline]
    pub fn into_context(self) -> T::Context {
        let mut me = ManuallyDrop::new(self);
        unsafe { me.deallocate() }
    }

    #[inline]
    #[must_use]
    pub unsafe fn from_raw_parts(ptr: *mut u8, capacity: usize) -> Self {
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        unsafe { Self::from_nonnull(ptr, capacity) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn from_nonnull(ptr: NonNull<u8>, capacity: usize) -> Self {
        Self {
            ptr,
            capacity,
            _marker: BufferDropCheck::default(),
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let context = self.ptr_to_context();
        unsafe { context.as_ref_unchecked() }
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, MutPtrs<'_, T>) {
        let Self { ptr, capacity, .. } = *self;
        let context = self.context();
        let ptr = ptr.as_ptr();

        let ptrs = unsafe { ptrs_from_buffer_mut::<T>(context, ptr, capacity) };
        (context, ptrs)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
    }

    #[inline]
    pub fn are_fields_dangling(&self) -> bool {
        let context = self.context();
        let capacity = self.capacity();

        let layout = unsafe { context.buffer_layout(capacity).unwrap_unchecked() };
        layout_is_dangling(layout)
    }

    #[inline]
    fn current_memory(&self, context: &T::Context) -> Option<(NonNull<u8>, Layout)> {
        let Self { ptr, capacity, .. } = *self;

        let layout = unsafe { buffer_layout::<T>(context, capacity).unwrap_unchecked() };
        if layout_is_dangling(layout) {
            return None;
        }

        Some((ptr, layout))
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
            T: AllocSoa + ?Sized,
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
        let Self { capacity, .. } = self;
        additional > capacity.wrapping_sub(len)
    }

    #[inline]
    unsafe fn set_ptr_and_capacity(&mut self, ptr: NonNull<u8>, capacity: usize) {
        self.ptr = ptr;
        self.capacity = capacity;
        self.set_capacity_in_buffer();
    }

    fn grow_amortized(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        debug_assert!(additional > 0);

        let required_capacity = len.checked_add(additional).ok_or(CapacityOverflow)?;
        let capacity = usize::max(self.capacity().saturating_mul(2), required_capacity);

        let context = self.context();
        let capacity = usize::max(Self::min_non_zero_cap(context), capacity);
        let (new_layout, capacity) =
            buffer_layout_capacity::<T>(context, capacity).map_err(|_| CapacityOverflow)?;

        let current_memory = self.current_memory(context);
        let ptr = unsafe { finish_grow(new_layout, current_memory)? };

        unsafe { self.set_ptr_and_capacity(ptr, capacity) }
        Ok(())
    }

    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        let capacity = len.checked_add(additional).ok_or(CapacityOverflow)?;

        let context = self.context();
        let (new_layout, capacity) =
            buffer_layout_capacity::<T>(context, capacity).map_err(|_| CapacityOverflow)?;

        let current_memory = self.current_memory(context);
        let ptr = unsafe { finish_grow(new_layout, current_memory)? };

        unsafe { self.set_ptr_and_capacity(ptr, capacity) }
        Ok(())
    }

    fn shrink(&mut self, capacity: usize) -> Result<(), TryReserveError> {
        assert!(
            capacity <= self.capacity(),
            "tried to shrink to a larger capacity",
        );

        let context = self.context();
        let Some((ptr, old_layout)) = self.current_memory(context) else {
            return Ok(());
        };

        let Ok((new_layout, capacity)) = buffer_layout_capacity::<T>(context, capacity) else {
            return Err(CapacityOverflow.into());
        };
        if layout_is_dangling(new_layout) {
            unsafe { dealloc(ptr.as_ptr(), old_layout) }

            let ptr = new_layout.dangling_ptr();
            unsafe { self.set_ptr_and_capacity(ptr, 0) }
            return Ok(());
        }

        let ptr = unsafe { realloc(ptr.as_ptr(), old_layout, new_layout.size()) };
        let Some(ptr) = NonNull::new(ptr) else {
            return Err(alloc_error(new_layout).into());
        };
        unsafe { self.set_ptr_and_capacity(ptr, capacity) }
        Ok(())
    }
}

impl<T> RawSoaVec<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    #[must_use]
    pub unsafe fn into_box(self, len: usize) -> Box<SoaSlice<T>> {
        debug_assert!(
            len <= self.capacity(),
            "`len` must be smaller than or equal to `self.capacity()`",
        );

        let me = ManuallyDrop::new(self);
        let slice = unsafe { slice_from_raw_parts_mut(me.as_ptr(), len, me.capacity()) };
        unsafe { Box::from_raw(slice) }
    }
}

impl<T> Drop for RawSoaVec<T>
where
    T: AllocSoa + ?Sized,
{
    #[inline]
    fn drop(&mut self) {
        let _ = unsafe { self.deallocate() };
    }
}

unsafe impl<T> Send for RawSoaVec<T>
where
    T: AllocSoa + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for RawSoaVec<T>
where
    T: AllocSoa + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}

#[inline(never)]
unsafe fn finish_grow(
    new_layout: Layout,
    current_memory: Option<(NonNull<u8>, Layout)>,
) -> Result<NonNull<u8>, TryReserveError> {
    if layout_is_dangling(new_layout) {
        return Err(CapacityOverflow.into());
    }

    let ptr = if let Some((ptr, old_layout)) = current_memory {
        debug_assert_eq!(old_layout.align(), new_layout.align());
        unsafe { realloc(ptr.as_ptr(), old_layout, new_layout.size()) }
    } else {
        unsafe { alloc(new_layout) }
    };

    let Some(ptr) = NonNull::new(ptr) else {
        return Err(alloc_error(new_layout).into());
    };
    Ok(ptr)
}

#[cfg_attr(not(panic = "immediate-abort"), inline(never))]
fn capacity_overflow() -> ! {
    panic!("capacity overflow")
}

#[cold]
#[expect(clippy::needless_pass_by_value)]
fn handle_error(error: TryReserveError) -> ! {
    match error.kind() {
        CapacityOverflow => capacity_overflow(),
        AllocError { layout, .. } => handle_alloc_error(layout),
    }
}
