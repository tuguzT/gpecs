use crate::{
    ptr,
    traits::{RawSoaContext, RefsMut, Soa, SoaContext, SoaRead, SoaWrite},
};

/// Version of [`core::mem::replace()`] but for [SoA](Soa) types.
pub fn replace<'a, T, R>(context: &T::Context, dest: RefsMut<'_, 'a, T>, src: T) -> R
where
    T: Soa<'a> + SoaRead<R> + SoaWrite,
{
    let dest = context.mut_refs_as_mut_ptrs(T::Context::upcast_mut_refs(dest));
    unsafe { ptr::replace(context, dest, src) }
}

/// Version of [`core::mem::swap()`] but for [SoA](Soa) types.
pub fn swap<'a, T>(context: &T::Context, x: RefsMut<'_, 'a, T>, y: RefsMut<'_, 'a, T>)
where
    T: Soa<'a> + ?Sized,
{
    let x = context.mut_refs_as_mut_ptrs(T::Context::upcast_mut_refs(x));
    let y = context.mut_refs_as_mut_ptrs(T::Context::upcast_mut_refs(y));

    // SAFETY: `&mut` guarantees these are typed readable and writable
    // as well as non-overlapping.
    unsafe { context.ptrs_swap(x, y) }
}
