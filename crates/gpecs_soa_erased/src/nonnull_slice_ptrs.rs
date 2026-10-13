use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::NonNull,
};

use crate::{
    BufferOffsetsFrom, BufferOffsetsFromSelf, BufferOffsetsOf, CovariantFieldLayouts,
    ErasedSoaMutPtrs, ErasedSoaMutSlicePtrs, ErasedSoaNonNullPtrs, ErasedSoaNonNullPtrsIter,
    ErasedSoaPtrs, ErasedSoaSlicePtrs,
    data::ErasedNonNullSlicePtr,
    error::{
        DowncastError, SlicePtrsError, check_offset, check_offset_len, check_ptr_align,
        check_sufficient_align, check_sufficient_len,
    },
    ptr::slice::{ConstPtr, MutPtr, NonNullSliceItemPtr, SliceItemPtr},
    soa::{
        field::{
            FieldLayouts, FieldLayoutsItem, FieldLayoutsOutput, FieldLayoutsOwned, buffer_offsets,
        },
        layout::WithLayout,
        traits::{SliceNonNullPtrs, SoaAlloc, SoaRawContext},
    },
};

pub struct ErasedSoaNonNullSlicePtrs<D, P>
where
    D: ?Sized,
    P: SliceItemPtr,
{
    len: usize,
    ptrs: ErasedSoaNonNullPtrs<D, P>,
}

impl<D, P> ErasedSoaNonNullSlicePtrs<D, P>
where
    P: SliceItemPtr,
{
    #[inline]
    pub unsafe fn from_parts(
        layouts: D,
        buffer: NonNull<[P::Item]>,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaNonNullPtrs::from_parts(layouts, buffer, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaNonNullPtrs<D, P>, len: usize) -> Self {
        Self { len, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, NonNull<[P::Item]>, usize, usize, usize) {
        let Self { ptrs, len } = self;
        let (layouts, buffer, capacity, offset) = ptrs.into_parts();
        (layouts, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_nonnull_ptrs(self) -> ErasedSoaNonNullPtrs<D, P> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub unsafe fn map_layouts<N, F>(self, f: F) -> ErasedSoaNonNullSlicePtrs<N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self { ptrs, len } = self;

        let ptrs = unsafe { ptrs.map_layouts(f) };
        unsafe { ErasedSoaNonNullSlicePtrs::from_ptrs(ptrs, len) }
    }
}

impl<D, P> ErasedSoaNonNullSlicePtrs<D, P>
where
    D: ?Sized,
    P: SliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
        let Self { ptrs, .. } = self;
        ptrs.as_buffer()
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
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<D, P> ErasedSoaNonNullSlicePtrs<D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, ConstPtr<P::Ptrs>> {
        let Self { ptrs, .. } = self;
        ptrs.into_ptrs()
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaMutPtrs<D, MutPtr<P::Ptrs>> {
        let Self { ptrs, .. } = self;
        ptrs.into_mut_ptrs()
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> ErasedSoaSlicePtrs<D, ConstPtr<P::Ptrs>> {
        let Self { ptrs, len } = self;
        let ptrs = ptrs.into_ptrs();
        unsafe { ErasedSoaSlicePtrs::from_ptrs(ptrs, len) }
    }

    #[inline]
    pub fn into_mut_slice_ptrs(self) -> ErasedSoaMutSlicePtrs<D, MutPtr<P::Ptrs>> {
        let Self { ptrs, len } = self;
        let ptrs = ptrs.into_mut_ptrs();
        unsafe { ErasedSoaMutSlicePtrs::from_ptrs(ptrs, len) }
    }
}

impl<D, P> ErasedSoaNonNullSlicePtrs<D, P>
where
    D: FieldLayoutsOwned,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn new(
        layouts: D,
        buffer: NonNull<[P::Item]>,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, SlicePtrsError> {
        check_offset(offset, capacity)?;
        check_offset_len(offset, len, capacity)?;

        let mut offsets = buffer_offsets(layouts.field_layouts(), capacity);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<P::Item>())
                .map_err(SlicePtrsError::from)
        })?;

        let layout = offsets.into_buffer().layout();
        check_ptr_align(buffer.as_ptr().cast(), layout)?;

        let buffer_layout = Layout::array::<P::Item>(buffer.len())?;
        check_sufficient_len(buffer_layout.size(), layout.size())?;

        let me = unsafe { Self::from_parts(layouts, buffer, capacity, offset, len) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<SliceNonNullPtrs<'_, T>, DowncastError<Self>>
    where
        T: SoaAlloc + ?Sized,
    {
        let Self { ptrs, len } = self;

        let result = unsafe { ptrs.downcast::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let slices = context.nonnull_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<'a, D, P> ErasedSoaNonNullSlicePtrs<D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedSoaNonNullSlicePtrsIter<D::OutputIter, P, BufferOffsetsOf<D::OutputItem>> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaNonNullSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D, P> Debug for ErasedSoaNonNullSlicePtrs<D, P>
where
    D: Debug + ?Sized,
    P: SliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, ptrs } = self;
        f.debug_struct("ErasedSoaSliceNonNullPtrs")
            .field("len", len)
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaNonNullSlicePtrs<D, P>
where
    D: Clone,
    P: SliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { len, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { len, ptrs }
    }
}

impl<D, P> Copy for ErasedSoaNonNullSlicePtrs<D, P>
where
    D: Copy,
    P: SliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaNonNullSlicePtrs<D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullSlicePtr<P>;
    type IntoIter = ErasedSoaNonNullSlicePtrsIter<D::OutputIter, P, BufferOffsetsOf<D::OutputItem>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedSoaNonNullSlicePtrs<D, P>
where
    D: IntoIterator<Item: WithLayout + BufferOffsetsFromSelf>,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullSlicePtr<P>;
    type IntoIter = ErasedSoaNonNullSlicePtrsIter<D::IntoIter, P, BufferOffsetsOf<D::Item>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, len } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaNonNullSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaNonNullSlicePtrs<D, P>
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

impl<D, P> CovariantFieldLayouts for ErasedSoaNonNullSlicePtrs<D, P>
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

pub struct ErasedSoaNonNullSlicePtrsIter<D, P, F>
where
    D: ?Sized,
    P: SliceItemPtr,
{
    len: usize,
    ptrs: ErasedSoaNonNullPtrsIter<D, P, F>,
}

impl<D, P, F> ErasedSoaNonNullSlicePtrsIter<D, P, F>
where
    P: SliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        ptrs: ErasedSoaNonNullPtrsIter<D, P, F>,
        len: usize,
    ) -> Self {
        Self { len, ptrs }
    }
}

impl<D, P, F> ErasedSoaNonNullSlicePtrsIter<D, P, F>
where
    D: ?Sized,
    P: SliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
        let Self { ptrs, .. } = self;
        ptrs.as_buffer()
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
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.layouts()
    }
}

impl<'a, D, P, F> ErasedSoaNonNullSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaNonNullSlicePtrsIter<D::OutputIter, P, F> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaNonNullSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedSoaNonNullSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    type Item = ErasedNonNullSlicePtr<P>;
    type IntoIter = ErasedSoaNonNullSlicePtrsIter<D::OutputIter, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedSoaNonNullSlicePtrsIter<D, P, F>
where
    D: FieldLayoutsOwned + ?Sized,
    P: NonNullSliceItemPtr + Debug,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<D, P, F> Clone for ErasedSoaNonNullSlicePtrsIter<D, P, F>
where
    D: Clone,
    P: SliceItemPtr,
    F: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { len, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { len, ptrs }
    }
}

impl<D, P, F> Iterator for ErasedSoaNonNullSlicePtrsIter<D, P, F>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedNonNullSlicePtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ref mut ptrs, len } = *self;

        let data = ptrs.next()?;
        let item = unsafe { ErasedNonNullSlicePtr::from_parts(data, len) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, P, F> ExactSizeIterator for ErasedSoaNonNullSlicePtrsIter<D, P, F>
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

impl<D, P, F> FusedIterator for ErasedSoaNonNullSlicePtrsIter<D, P, F>
where
    D: FusedIterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedSoaNonNullSlicePtrsIter<D, P, F>
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

impl<D, P, F> CovariantFieldLayouts for ErasedSoaNonNullSlicePtrsIter<D, P, F>
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
