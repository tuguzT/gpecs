use crate::{
    ptr,
    traits::{RawSoaContext, RefsMut, Soa, SoaContext, SoaRead, SoaWrite},
};

/// Version of [`core::mem::replace()`] but for [SoA](Soa) types.
pub fn replace<'a, 'data, T, R, W>(
    context: &'a T::Context,
    dest: RefsMut<'a, 'data, T>,
    src: W,
) -> R
where
    T: Soa<'data> + SoaRead<'a, R> + SoaWrite<W> + ?Sized,
{
    let dest = context.mut_refs_as_mut_ptrs(dest);
    unsafe { ptr::replace::<T, R, W>(context, dest, src) }
}

/// Version of [`core::mem::swap()`] but for [SoA](Soa) types.
pub fn swap<'data, T>(context: &T::Context, x: RefsMut<'_, 'data, T>, y: RefsMut<'_, 'data, T>)
where
    T: Soa<'data> + ?Sized,
{
    let x = context.mut_refs_as_mut_ptrs(T::Context::upcast_mut_refs(x));
    let y = context.mut_refs_as_mut_ptrs(T::Context::upcast_mut_refs(y));

    // SAFETY: `&mut` guarantees these are typed readable and writable
    // as well as non-overlapping.
    unsafe { context.ptrs_swap_nonoverlapping(x, y, 1) }
}
