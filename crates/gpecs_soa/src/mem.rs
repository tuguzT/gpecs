use crate::traits::{RawSoaContext, Soa, SoaRead, SoaWrite};

/// Version of [`core::mem::replace()`] but for [`Soa`] references.
pub fn replace<T>(context: &T::Context, dest: T::RefsMut<'_, '_>, src: T) -> T
where
    T: Soa + SoaRead + SoaWrite,
{
    let dest = T::refs_mut_as_ptrs(context, T::upcast_refs_mut(dest));

    // SAFETY: We read from `dest` but directly write `src` into it afterwards,
    // such that the old value is not duplicated. Nothing is dropped and
    // nothing here can panic.
    unsafe {
        let result = T::read(context, context.ptrs_cast_const(dest.clone()));
        T::write(context, dest, src);
        result
    }
}

/// Version of [`core::mem::swap()`] but for [`Soa`] references.
pub fn swap<T>(context: &T::Context, x: T::RefsMut<'_, '_>, y: T::RefsMut<'_, '_>)
where
    T: Soa + ?Sized,
{
    let x = T::refs_mut_as_ptrs(context, T::upcast_refs_mut(x));
    let y = T::refs_mut_as_ptrs(context, T::upcast_refs_mut(y));

    // SAFETY: `&mut` guarantees these are typed readable and writable
    // as well as non-overlapping.
    unsafe { context.ptrs_swap(x, y) }
}
