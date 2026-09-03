use core::mem::ManuallyDrop;

use crate::traits::{MutPtrs, RawSoaContext, ReadSoaContext, SoaRead, SoaWrite, WriteSoaContext};

/// Version of [`core::ptr::replace()`] but for [SoA](crate::traits::RawSoa) types.
pub unsafe fn replace<'a, T, R, W>(context: &'a T::Context, dst: MutPtrs<'a, T>, src: W) -> R
where
    T: SoaRead<'a, R> + SoaWrite<W> + ?Sized,
{
    let result = unsafe {
        let src = context.ptrs_cast_const(dst.clone());
        context.ptrs_read(src)
    };
    let result = ManuallyDrop::new(result);

    unsafe { context.ptrs_write(dst, src) }
    ManuallyDrop::into_inner(result)
}
