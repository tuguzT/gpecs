use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    CovariantFieldLayouts, ErasedSoaNonNullPtrs, ErasedSoaNonNullSlicePtrs,
    ErasedSoaNonNullSlicePtrsIter, ErasedSoaPtrs, ErasedSoaSlicePtrs,
    data::ErasedSlice,
    error::{DowncastError, SlicePtrsError},
    layout::WithLayout,
    offsets::{BufferOffsetsFrom, BufferOffsetsFromSelf, BufferOffsetsOf},
    ptr::slice::{ConstPtr, NonNullSliceItemPtr, SliceItemPtr},
    soa::{
        field::{FieldLayouts, FieldLayoutsItem, FieldLayoutsOutput, FieldLayoutsOwned},
        traits::{Slices, Soa, SoaAlloc, SoaContext, SoaRawContext},
    },
};

pub struct ErasedSoaSlices<'a, D, P>
where
    D: ?Sized,
    P: SliceItemPtr,
{
    phantom: PhantomData<&'a [P::Item]>,
    ptrs: ErasedSoaNonNullSlicePtrs<D, P>,
}

impl<'a, D, P> ErasedSoaSlices<'a, D, P>
where
    P: SliceItemPtr,
{
    #[inline]
    pub unsafe fn from_parts(
        layouts: D,
        buffer: &'a [P::Item],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let buffer = NonNull::from_ref(buffer);
        let ptrs = unsafe {
            ErasedSoaNonNullSlicePtrs::from_parts(layouts, buffer, capacity, offset, len)
        };
        unsafe { Self::from_nonnull_slice_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_nonnull_slice_ptrs(ptrs: ErasedSoaNonNullSlicePtrs<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub unsafe fn from_nonnull_ptrs(ptrs: ErasedSoaNonNullPtrs<D, P>, len: usize) -> Self {
        let ptrs = unsafe { ErasedSoaNonNullSlicePtrs::from_ptrs(ptrs, len) };
        unsafe { Self::from_nonnull_slice_ptrs(ptrs) }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a [P::Item], usize, usize, usize) {
        let Self { ptrs, .. } = self;
        let (layouts, buffer, capacity, offset, len) = ptrs.into_parts();

        let buffer = unsafe { buffer.as_ref() };
        (layouts, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_nonnull_slice_ptrs(self) -> ErasedSoaNonNullSlicePtrs<D, P> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub fn into_nonnull_ptrs(self) -> ErasedSoaNonNullPtrs<D, P> {
        let Self { ptrs, .. } = self;
        ptrs.into_ptrs()
    }

    #[inline]
    pub unsafe fn map_layouts<N, F>(self, f: F) -> ErasedSoaSlices<'a, N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self { ptrs, .. } = self;

        let ptrs = unsafe { ptrs.map_layouts(f) };
        unsafe { ErasedSoaSlices::from_nonnull_slice_ptrs(ptrs) }
    }
}

impl<D, P> ErasedSoaSlices<'_, D, P>
where
    D: ?Sized,
    P: SliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { buffer.as_ref() }
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

impl<D, P> ErasedSoaSlices<'_, D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_slice_ptrs(ptrs: ErasedSoaSlicePtrs<D, ConstPtr<P::Ptrs>>) -> Option<Self> {
        let ptrs = ptrs.cast_nonnull()?;
        let me = unsafe { Self::from_nonnull_slice_ptrs(ptrs) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_slice_ptrs_unchecked(
        ptrs: ErasedSoaSlicePtrs<D, ConstPtr<P::Ptrs>>,
    ) -> Self {
        let ptrs = unsafe { ptrs.cast_nonnull_unchecked() };
        unsafe { Self::from_nonnull_slice_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaPtrs<D, ConstPtr<P::Ptrs>>, len: usize) -> Option<Self> {
        let ptrs = ptrs.cast_nonnull()?;
        let me = unsafe { Self::from_nonnull_ptrs(ptrs, len) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_ptrs_unchecked(
        ptrs: ErasedSoaPtrs<D, ConstPtr<P::Ptrs>>,
        len: usize,
    ) -> Self {
        let ptrs = unsafe { ptrs.cast_nonnull_unchecked() };
        unsafe { Self::from_nonnull_ptrs(ptrs, len) }
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> ErasedSoaSlicePtrs<D, ConstPtr<P::Ptrs>> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, ConstPtr<P::Ptrs>> {
        let Self { ptrs, .. } = self;
        ptrs.into_ptrs().cast_const()
    }
}

impl<'a, D, P> ErasedSoaSlices<'a, D, P>
where
    D: FieldLayoutsOwned,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn new(
        layouts: D,
        buffer: &'a [P::Item],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, SlicePtrsError> {
        let buffer = NonNull::from_ref(buffer);
        let ptrs = ErasedSoaNonNullSlicePtrs::new(layouts, buffer, capacity, offset, len)?;

        let me = unsafe { Self::from_nonnull_slice_ptrs(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<Slices<'_, 'a, T>, DowncastError<Self>>
    where
        T: SoaAlloc + Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.downcast::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_nonnull_slice_ptrs(ptrs) };
        let slices = result.map_err(|err| err.map_value(into_self))?;

        // TODO: add SoaContext method to deref nonnull slice pointers
        let slices = context.nonnull_slice_ptrs_as_slice_ptrs(slices);
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Ok(slices)
    }
}

impl<'a, D, P> ErasedSoaSlices<'_, D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedSoaSlicesIter<'a, D::OutputIter, P, BufferOffsetsOf<D::OutputItem>> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicesIter::from_inner(ptrs) }
    }
}

impl<D, P> Debug for ErasedSoaSlices<'_, D, P>
where
    D: Debug + ?Sized,
    P: SliceItemPtr,
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
    P: SliceItemPtr,
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
    P: SliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaSlices<'_, D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedSlice<'a, P>;
    type IntoIter = ErasedSoaSlicesIter<'a, D::OutputIter, P, BufferOffsetsOf<D::OutputItem>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for ErasedSoaSlices<'a, D, P>
where
    D: IntoIterator<Item: WithLayout + BufferOffsetsFromSelf>,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedSlice<'a, P>;
    type IntoIter = ErasedSoaSlicesIter<'a, D::IntoIter, P, BufferOffsetsOf<D::Item>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaSlicesIter::from_inner(ptrs) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaSlices<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: SliceItemPtr,
{
    type Output = D::Output;
    type OutputIter = D::OutputIter;
    type OutputItem = D::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedSoaSlices<'_, D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: SliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedSoaSlicesIter<'a, D, P, F>
where
    D: ?Sized,
    P: SliceItemPtr,
{
    phantom: PhantomData<&'a [P::Item]>,
    ptrs: ErasedSoaNonNullSlicePtrsIter<D, P, F>,
}

impl<D, P, F> ErasedSoaSlicesIter<'_, D, P, F>
where
    P: SliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(ptrs: ErasedSoaNonNullSlicePtrsIter<D, P, F>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, P, F> ErasedSoaSlicesIter<'_, D, P, F>
where
    D: ?Sized,
    P: SliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { buffer.as_ref() }
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

impl<'a, D, P, F> ErasedSoaSlicesIter<'_, D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaSlicesIter<'a, D::OutputIter, P, F> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicesIter::from_inner(ptrs) }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedSoaSlicesIter<'_, D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    type Item = ErasedSlice<'a, P>;
    type IntoIter = ErasedSoaSlicesIter<'a, D::OutputIter, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedSoaSlicesIter<'_, D, P, F>
where
    D: FieldLayoutsOwned + ?Sized,
    P: NonNullSliceItemPtr<Item: Debug>,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<D, P, F> Clone for ErasedSoaSlicesIter<'_, D, P, F>
where
    D: Clone,
    P: SliceItemPtr,
    F: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { phantom, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { phantom, ptrs }
    }
}

impl<'a, D, P, F> Iterator for ErasedSoaSlicesIter<'a, D, P, F>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedSlice<'a, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ptrs, .. } = self;

        let item = unsafe { ptrs.next()?.as_ref() };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, P, F> ExactSizeIterator for ErasedSoaSlicesIter<'_, D, P, F>
where
    D: ExactSizeIterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P, F> FusedIterator for ErasedSoaSlicesIter<'_, D, P, F>
where
    D: FusedIterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedSoaSlicesIter<'_, D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: SliceItemPtr,
{
    type Output = D::Output;
    type OutputIter = D::OutputIter;
    type OutputItem = D::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_layouts()
    }
}

impl<D, P, F> CovariantFieldLayouts for ErasedSoaSlicesIter<'_, D, P, F>
where
    D: CovariantFieldLayouts + ?Sized,
    P: SliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
