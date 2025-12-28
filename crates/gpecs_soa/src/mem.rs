use crate::{
    ptr,
    traits::{RawSoaContext, Soa, SoaRead, SoaWrite},
};

/// Version of [`core::mem::replace()`] but for [SoA](Soa) types.
pub fn replace<T>(context: &T::Context, dest: T::RefsMut<'_, '_>, src: T) -> T
where
    T: Soa + SoaRead + SoaWrite,
{
    let dest = T::refs_mut_as_ptrs(context, T::upcast_refs_mut(dest));
    unsafe { ptr::replace(context, dest, src) }
}

/// Version of [`core::mem::swap()`] but for [SoA](Soa) types.
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
