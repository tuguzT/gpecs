use crate::traits::{MutPtrs, RawSoaContext, ReadSoaContext, SoaRead, SoaWrite, WriteSoaContext};

/// Version of [`core::ptr::replace()`] but for [SoA](crate::traits::RawSoa) types.
pub unsafe fn replace<'a, T, R, W>(context: &'a T::Context, dest: MutPtrs<'a, T>, src: W) -> R
where
    T: SoaRead<'a, R> + SoaWrite<W> + ?Sized,
{
    // SAFETY: We read from `dest` but directly write `src` into it afterwards,
    // such that the old value is not duplicated. Nothing is dropped and
    // nothing here can panic.
    unsafe {
        let result = context.read(context.ptrs_cast_const(dest.clone()));
        context.write(dest, src);
        result
    }
}
