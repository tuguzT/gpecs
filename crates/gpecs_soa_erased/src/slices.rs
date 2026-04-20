use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    CovariantFieldLayouts, ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter,
    data::ErasedSlice,
    error::{DowncastError, SlicePtrsError},
    ptr::slice::ConstSliceItemPtr,
    soa::{
        field::{FieldLayouts, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned},
        layout::WithLayout,
        traits::{AllocSoa, Slices, Soa, SoaContext},
    },
};

pub struct ErasedSoaSlices<'a, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    phantom: PhantomData<&'a [P::Item]>,
    ptrs: ErasedSoaSlicePtrs<D, P>,
}

impl<'a, D, P> ErasedSoaSlices<'a, D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        layouts: D,
        buffer: &'a [P::Item],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs =
            unsafe { ErasedSoaSlicePtrs::new_unchecked(layouts, buffer, capacity, offset, len) };
        unsafe { Self::from_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaSlicePtrs<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a [P::Item], usize, usize, usize) {
        let Self { ptrs, .. } = self;
        let (layouts, buffer, capacity, offset, len) = ptrs.into_parts();

        let buffer = unsafe { buffer.as_ref_unchecked() };
        (layouts, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaSlicePtrs<D, P> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub unsafe fn map_layouts<N, F>(self, f: F) -> ErasedSoaSlices<'a, N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self { ptrs, .. } = self;

        let ptrs = unsafe { ptrs.map_layouts(f) };
        unsafe { ErasedSoaSlices::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> ErasedSoaSlices<'a, D, P>
where
    D: FieldLayoutsOwned,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn new(
        layouts: D,
        buffer: &'a [P::Item],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, SlicePtrsError> {
        let ptrs = ErasedSoaSlicePtrs::new(layouts, buffer, capacity, offset, len)?;

        let me = unsafe { Self::from_ptrs(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<Slices<'_, 'a, T>, DowncastError<Self>>
    where
        T: AllocSoa + Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.downcast::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = result.map_err(|err| err.map_value(into_self))?;

        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        Ok(slices)
    }
}

impl<D, P> ErasedSoaSlices<'_, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { buffer.as_ref_unchecked() }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.offset()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.layouts()
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, D, P> ErasedSoaSlices<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaSlicesIter<'a, FieldLayoutsIter<'a, D>, P> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicesIter::from_ptrs(ptrs) }
    }
}

impl<D, P> Debug for ErasedSoaSlices<'_, D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_struct("ErasedSoaSlices")
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaSlices<'_, D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { phantom, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { phantom, ptrs }
    }
}

impl<D, P> Copy for ErasedSoaSlices<'_, D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaSlices<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedSlice<'a, P>;
    type IntoIter = ErasedSoaSlicesIter<'a, FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for ErasedSoaSlices<'a, D, P>
where
    D: IntoIterator<Item: WithLayout>,
    P: ConstSliceItemPtr,
{
    type Item = ErasedSlice<'a, P>;
    type IntoIter = ErasedSoaSlicesIter<'a, D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaSlicesIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaSlices<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedSoaSlices<'_, D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedSoaSlicesIter<'a, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    phantom: PhantomData<&'a [P::Item]>,
    ptrs: ErasedSoaSlicePtrsIter<D, P>,
}

impl<D, P> ErasedSoaSlicesIter<'_, D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_ptrs(ptrs: ErasedSoaSlicePtrsIter<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, P> ErasedSoaSlicesIter<'_, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { buffer.as_ref_unchecked() }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.offset()
    }

    #[inline]
    pub fn slice_len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.slice_len()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.layouts()
    }
}

impl<'a, D, P> ErasedSoaSlicesIter<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaSlicesIter<'a, FieldLayoutsIter<'a, D>, P> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicesIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaSlicesIter<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedSlice<'a, P>;
    type IntoIter = ErasedSoaSlicesIter<'a, FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedSoaSlicesIter<'_, D, P>
where
    D: FieldLayoutsOwned + ?Sized,
    P: ConstSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedSoaSlicesIter<'_, D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { phantom, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { phantom, ptrs }
    }
}

impl<'a, D, P> Iterator for ErasedSoaSlicesIter<'a, D, P>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedSlice<'a, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ptrs, .. } = self;

        let item = unsafe { ptrs.next()?.as_ref_unchecked() };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedSoaSlicesIter<'_, D, P>
where
    D: ExactSizeIterator<Item: WithLayout> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P> FusedIterator for ErasedSoaSlicesIter<'_, D, P>
where
    D: FusedIterator<Item: WithLayout> + ?Sized,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaSlicesIter<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedSoaSlicesIter<'_, D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
