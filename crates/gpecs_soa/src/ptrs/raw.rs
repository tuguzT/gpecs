use crate::traits::{AllocSoa, AllocSoaContext, MutPtrs, Ptrs};

#[inline]
pub unsafe fn from_buffer<T>(
    context: &T::Context,
    buffer: *const u8,
    capacity: usize,
) -> Ptrs<'_, T>
where
    T: AllocSoa + ?Sized,
{
    unsafe { context.ptrs_from_buffer(buffer, capacity) }
}

#[inline]
pub unsafe fn from_buffer_mut<T>(
    context: &T::Context,
    buffer: *mut u8,
    capacity: usize,
) -> MutPtrs<'_, T>
where
    T: AllocSoa + ?Sized,
{
    unsafe { context.mut_ptrs_from_buffer(buffer, capacity) }
}

#[inline]
pub unsafe fn copy_forward<T>(
    context: &T::Context,
    src: Ptrs<'_, T>,
    dst: MutPtrs<'_, T>,
    count: usize,
) where
    T: AllocSoa + ?Sized,
{
    unsafe { context.ptrs_copy_forward(src, dst, count) }
}

#[inline]
pub unsafe fn copy_backward<T>(
    context: &T::Context,
    src: Ptrs<'_, T>,
    dst: MutPtrs<'_, T>,
    count: usize,
) where
    T: AllocSoa + ?Sized,
{
    unsafe { context.ptrs_copy_backward(src, dst, count) }
}
