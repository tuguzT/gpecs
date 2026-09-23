use crate::{
    slices::SlicesIndex,
    traits::{MutPtrs, Ptrs, SliceMutPtrs, SlicePtrs, Slices, SlicesMut, Soa, SoaContext},
};

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
pub fn as_ptrs<'a, 'data, T>(context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Ptrs<'a, T>
where
    T: Soa<'data> + ?Sized,
{
    context.slices_as_ptrs(slices)
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
pub fn mut_as_slice_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    slices: SlicesMut<'a, 'data, T>,
) -> SlicePtrs<'a, T>
where
    T: Soa<'data> + ?Sized,
{
    context.mut_slices_as_slice_ptrs(slices)
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
pub fn mut_as_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    slices: SlicesMut<'a, 'data, T>,
) -> Ptrs<'a, T>
where
    T: Soa<'data> + ?Sized,
{
    context.mut_slices_as_ptrs(slices)
}

#[inline]
pub fn as_mut_ptrs<'a, 'data, T>(
    context: &'a T::Context,
    slices: SlicesMut<'a, 'data, T>,
) -> MutPtrs<'a, T>
where
    T: Soa<'data> + ?Sized,
{
    context.mut_slices_as_mut_ptrs(slices)
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

#[inline]
pub fn get<'a, 'data, T, I>(
    context: &'a T::Context,
    slices: Slices<'a, 'data, T>,
    index: I,
) -> Option<I::Refs<'a>>
where
    T: Soa<'data> + ?Sized,
    I: SlicesIndex<'data, T>,
{
    index.get(context, slices)
}

#[inline]
pub fn index<'a, 'data, T, I>(
    context: &'a T::Context,
    slices: Slices<'a, 'data, T>,
    index: I,
) -> I::Refs<'a>
where
    T: Soa<'data> + ?Sized,
    I: SlicesIndex<'data, T>,
{
    index.index(context, slices)
}

#[inline]
pub fn get_mut<'a, 'data, T, I>(
    context: &'a T::Context,
    slices: SlicesMut<'a, 'data, T>,
    index: I,
) -> Option<I::RefsMut<'a>>
where
    T: Soa<'data> + ?Sized,
    I: SlicesIndex<'data, T>,
{
    index.get_mut(context, slices)
}

#[inline]
pub fn index_mut<'a, 'data, T, I>(
    context: &'a T::Context,
    slices: SlicesMut<'a, 'data, T>,
    index: I,
) -> I::RefsMut<'a>
where
    T: Soa<'data> + ?Sized,
    I: SlicesIndex<'data, T>,
{
    index.index_mut(context, slices)
}
