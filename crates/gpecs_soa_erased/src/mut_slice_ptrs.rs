use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    CovariantFieldLayouts, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter, ErasedSoaMutSlices,
    ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter, ErasedSoaSlices,
    data::{ErasedMutSlicePtr, ErasedSlicePtr},
    error::{DowncastError, SlicePtrsError, check_offset, check_offset_len},
    error::{check_ptr_align, check_sufficient_align, check_sufficient_len},
    layout::WithLayout,
    offsets::{BufferOffsetsFrom, BufferOffsetsFromLayout},
    ptr::slice::{CastConst, MutSliceItemPtr},
    soa::{
        field::{
            FieldLayouts, FieldLayoutsItem, FieldLayoutsIter, FieldLayoutsOutput,
            FieldLayoutsOwned, buffer_offsets,
        },
        traits::{AllocSoa, RawSoaContext, SliceMutPtrs},
    },
};

pub struct ErasedSoaMutSlicePtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    len: usize,
    ptrs: ErasedSoaMutPtrs<D, P>,
}

impl<D, P> ErasedSoaMutSlicePtrs<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        layouts: D,
        buffer: *mut [P::Item],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaMutPtrs::new_unchecked(layouts, buffer, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaMutPtrs<D, P>, len: usize) -> Self {
        Self { len, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *mut [P::Item], usize, usize, usize) {
        let Self { ptrs, len } = self;
        let (layouts, buffer, capacity, offset) = ptrs.into_parts();
        (layouts, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaMutPtrs<D, P> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub unsafe fn map_layouts<N, F>(self, f: F) -> ErasedSoaMutSlicePtrs<N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self { ptrs, len } = self;

        let ptrs = unsafe { ptrs.map_layouts(f) };
        unsafe { ErasedSoaMutSlicePtrs::from_ptrs(ptrs, len) }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSoaSlicePtrs<D, CastConst<P>> {
        let Self { ptrs, len } = self;
        let ptrs = ptrs.cast_const();
        unsafe { ErasedSoaSlicePtrs::from_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedSoaSlices<'a, D, CastConst<P>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedSoaMutSlices<'a, D, P> {
        unsafe { ErasedSoaMutSlices::from_ptrs(self) }
    }
}

impl<D, P> ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldLayoutsOwned,
    P: MutSliceItemPtr,
{
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        layouts: D,
        buffer: *mut [P::Item],
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
        check_ptr_align(buffer.cast(), layout)?;

        let buffer_layout = Layout::array::<P::Item>(buffer.len())?;
        check_sufficient_len(buffer_layout.size(), layout.size())?;

        let me = unsafe { Self::new_unchecked(layouts, buffer, capacity, offset, len) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<SliceMutPtrs<'_, T>, DowncastError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self { ptrs, len } = self;

        let result = unsafe { ptrs.downcast::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let slices = context.mut_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D, P> ErasedSoaMutSlicePtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { ptrs, .. } = self;
        ptrs.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [P::Item] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut_buffer()
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

impl<'a, D, P> ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedSoaSlicePtrsIter<FieldLayoutsIter<'a, D>, CastConst<P>, BufferOffsetsFromLayout>
    {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicePtrsIter::new_unchecked(ptrs, len) }
    }

    #[inline]
    pub fn iter_mut(
        &'a mut self,
    ) -> ErasedSoaMutSlicePtrsIter<FieldLayoutsIter<'a, D>, P, BufferOffsetsFromLayout> {
        let Self { ref mut ptrs, len } = *self;

        let ptrs = ptrs.iter_mut();
        unsafe { ErasedSoaMutSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D, P> Debug for ErasedSoaMutSlicePtrs<D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, ptrs } = self;
        f.debug_struct("ErasedSoaSliceMutPtrs")
            .field("len", len)
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaMutSlicePtrs<D, P>
where
    D: Clone,
    P: MutSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { len, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { len, ptrs }
    }
}

impl<D, P> Copy for ErasedSoaMutSlicePtrs<D, P>
where
    D: Copy,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedSlicePtr<CastConst<P>>;
    type IntoIter =
        ErasedSoaSlicePtrsIter<FieldLayoutsIter<'a, D>, CastConst<P>, BufferOffsetsFromLayout>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for &'a mut ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutSlicePtr<P>;
    type IntoIter = ErasedSoaMutSlicePtrsIter<FieldLayoutsIter<'a, D>, P, BufferOffsetsFromLayout>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D, P> IntoIterator for ErasedSoaMutSlicePtrs<D, P>
where
    D: IntoIterator<Item: WithLayout>,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutSlicePtr<P>;
    type IntoIter = ErasedSoaMutSlicePtrsIter<D::IntoIter, P, BufferOffsetsFromLayout>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, len } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaMutSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedSoaMutSlicePtrs<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    len: usize,
    ptrs: ErasedSoaMutPtrsIter<D, P, F>,
}

impl<D, P, F> ErasedSoaMutSlicePtrsIter<D, P, F>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(ptrs: ErasedSoaMutPtrsIter<D, P, F>, len: usize) -> Self {
        Self { len, ptrs }
    }
}

impl<D, P, F> ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { ptrs, .. } = self;
        ptrs.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [P::Item] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut_buffer()
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

impl<'a, D, P, F> ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaMutSlicePtrsIter<FieldLayoutsIter<'a, D>, P, F> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaMutSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    type Item = ErasedMutSlicePtr<P>;
    type IntoIter = ErasedSoaMutSlicePtrsIter<FieldLayoutsIter<'a, D>, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: FieldLayoutsOwned + ?Sized,
    P: MutSliceItemPtr + Debug,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<D, P, F> Clone for ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: Clone,
    P: MutSliceItemPtr,
    F: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { len, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { len, ptrs }
    }
}

impl<D, P, F> Iterator for ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedMutSlicePtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ref mut ptrs, len } = *self;

        let data = ptrs.next()?;
        let item = unsafe { ErasedMutSlicePtr::from_parts(data, len) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, P, F> ExactSizeIterator for ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: ExactSizeIterator<Item: WithLayout> + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P, F> FusedIterator for ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: FusedIterator<Item: WithLayout> + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_layouts()
    }
}

impl<D, P, F> CovariantFieldLayouts for ErasedSoaMutSlicePtrsIter<D, P, F>
where
    D: CovariantFieldLayouts + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
