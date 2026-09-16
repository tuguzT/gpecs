use crate::traits::{SliceMutPtrs, SlicePtrs, Slices, SlicesMut, Soa, SoaContext};

#[inline]
pub fn upcast<'short, 'long: 'short, 'data, T>(
    from: Slices<'long, 'data, T>,
) -> Slices<'short, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    T::Context::slices_upcast(from)
}

#[inline]
pub unsafe fn from_slice_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    slices: SlicePtrs<'a, T>,
) -> Slices<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    unsafe { context.slices_from_slice_ptrs(slices) }
}

#[inline]
pub fn as_slice_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    slices: Slices<'a, 'data, T>,
) -> SlicePtrs<'a, T>
where
    T: Soa<'data> + ?Sized,
{
    context.slices_as_slice_ptrs(slices)
}

#[inline]
pub fn len<'data, T>(context: &T::Context, slices: &Slices<'_, 'data, T>) -> usize
where
    T: Soa<'data> + ?Sized,
{
    context.slices_len(slices)
}

#[inline]
pub fn upcast_mut<'short, 'long: 'short, 'data, T>(
    from: SlicesMut<'long, 'data, T>,
) -> SlicesMut<'short, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    T::Context::mut_slices_upcast(from)
}

#[inline]
pub unsafe fn from_mut_slice_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
) -> SlicesMut<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
}

#[inline]
pub fn as_mut_slice_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    slices: SlicesMut<'a, 'data, T>,
) -> SliceMutPtrs<'a, T>
where
    T: Soa<'data> + ?Sized,
{
    context.mut_slices_as_mut_slice_ptrs(slices)
}

#[inline]
pub fn len_mut<'data, T>(context: &T::Context, slices: &SlicesMut<'_, 'data, T>) -> usize
where
    T: Soa<'data> + ?Sized,
{
    context.mut_slices_len(slices)
}

#[inline]
pub fn as_slices<'a, 'data, T>(
    context: &'a T::Context,
    slices: SlicesMut<'a, 'data, T>,
) -> Slices<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    context.mut_slices_as_slices(slices)
}
