use core::ptr::NonNull;

use crate::traits::{MutPtrs, NonNullPtrs, Ptrs, SoaAlloc, SoaAllocContext};

#[inline]
pub unsafe fn from_buffer<T>(
    context: &T::Context,
    buffer: *const u8,
    capacity: usize,
) -> Ptrs<'_, T>
where
    T: SoaAlloc + ?Sized,
{
    unsafe { context.ptrs_from_buffer(buffer, capacity) }
}

#[inline]
pub unsafe fn from_mut_buffer<T>(
    context: &T::Context,
    buffer: *mut u8,
    capacity: usize,
) -> MutPtrs<'_, T>
where
    T: SoaAlloc + ?Sized,
{
    unsafe { context.mut_ptrs_from_buffer(buffer, capacity) }
}

#[inline]
pub unsafe fn from_nonnull_buffer<T>(
    context: &T::Context,
    buffer: NonNull<u8>,
    capacity: usize,
) -> NonNullPtrs<'_, T>
where
    T: SoaAlloc + ?Sized,
{
    unsafe { context.nonnull_ptrs_from_buffer(buffer, capacity) }
}

#[inline]
pub unsafe fn copy_forward<T>(
    context: &T::Context,
    src: Ptrs<'_, T>,
    dst: MutPtrs<'_, T>,
    count: usize,
) where
    T: SoaAlloc + ?Sized,
{
    unsafe { context.ptrs_copy_forward(src, dst, count) }
}

#[inline]
pub unsafe fn copy_forward_nonnull<T>(
    context: &T::Context,
    src: NonNullPtrs<'_, T>,
    dst: NonNullPtrs<'_, T>,
    count: usize,
) where
    T: SoaAlloc + ?Sized,
{
    unsafe { context.nonnull_ptrs_copy_forward(src, dst, count) }
}

#[inline]
pub unsafe fn copy_backward<T>(
    context: &T::Context,
    src: Ptrs<'_, T>,
    dst: MutPtrs<'_, T>,
    count: usize,
) where
    T: SoaAlloc + ?Sized,
{
    unsafe { context.ptrs_copy_backward(src, dst, count) }
}

#[inline]
pub unsafe fn copy_backward_nonnull<T>(
    context: &T::Context,
    src: NonNullPtrs<'_, T>,
    dst: NonNullPtrs<'_, T>,
    count: usize,
) where
    T: SoaAlloc + ?Sized,
{
    unsafe { context.nonnull_ptrs_copy_backward(src, dst, count) }
}
