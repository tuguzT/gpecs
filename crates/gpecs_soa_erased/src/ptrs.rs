use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr,
};

use crate::{
    CovariantFieldLayouts, ErasedSoa, ErasedSoaMutPtrs, ErasedSoaRefs,
    assert::{assert_layouts, check_downcast},
    dangling::{Dangling, dangling},
    data::ErasedPtr,
    error::{
        DowncastError, FromFieldsLayoutsError, InsufficientAlignError, PtrsError, check_offset,
        check_ptr_align, check_sufficient_align, check_sufficient_len,
    },
    layout::{WithLayout, bytes_to_items},
    offsets::{BufferOffsetsFrom, BufferOffsetsFromSelf, BufferOffsetsOf},
    ptr::slice::{CastMut, ConstSliceItemPtr},
    soa::{
        field::{
            BufferOffset, FieldLayouts, FieldLayoutsItem, FieldLayoutsOutput, FieldLayoutsOwned,
            buffer_offsets,
        },
        traits::{AllocSoa, AllocSoaContext, Ptrs, RawSoaContext},
    },
    storage::AlignedStorageFromLayout,
};

pub struct ErasedSoaPtrs<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    buffer: *const [P::Item],
    capacity: usize,
    offset: usize,
    layouts: D,
}

impl<D, P> ErasedSoaPtrs<D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        layouts: D,
        buffer: *const [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Self {
        Self {
            buffer,
            capacity,
            offset,
            layouts,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *const [P::Item], usize, usize) {
        let Self {
            layouts,
            buffer,
            capacity,
            offset,
        } = self;
        (layouts, buffer, capacity, offset)
    }

    #[inline]
    pub unsafe fn map_layouts<N, F>(self, f: F) -> ErasedSoaPtrs<N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self {
            layouts,
            buffer,
            capacity,
            offset,
        } = self;

        let layouts = f(layouts);
        unsafe { ErasedSoaPtrs::new_unchecked(layouts, buffer, capacity, offset) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedSoaMutPtrs<D, CastMut<P>> {
        let Self {
            layouts,
            buffer,
            capacity,
            offset,
        } = self;

        let buffer = buffer.cast_mut();
        unsafe { ErasedSoaMutPtrs::new_unchecked(layouts, buffer, capacity, offset) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedSoaRefs<'a, D, P> {
        unsafe { ErasedSoaRefs::from_ptrs(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { offset, .. } = self;

        let offset = unsafe { offset.unchecked_add(count) };
        Self { offset, ..self }
    }
}

impl<D, P> ErasedSoaPtrs<D, P>
where
    D: FieldLayoutsOwned,
    P: ConstSliceItemPtr,
{
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        layouts: D,
        buffer: *const [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, PtrsError> {
        check_offset(offset, capacity)?;

        let mut offsets = buffer_offsets(layouts.field_layouts(), capacity);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<P::Item>())
                .map_err(PtrsError::from)
        })?;

        let layout = offsets.into_buffer().layout();
        check_ptr_align(buffer.cast(), layout)?;

        let buffer_layout = Layout::array::<P::Item>(buffer.len())?;
        check_sufficient_len(buffer_layout.size(), layout.size())?;

        let me = unsafe { Self::new_unchecked(layouts, buffer, capacity, offset) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(layouts: D) -> Result<Self, InsufficientAlignError> {
        let Dangling { addr, capacity } = dangling::<_, P::Item>(layouts.field_layouts())?;

        let data = ptr::without_provenance(addr);
        let buffer = ptr::slice_from_raw_parts(data, 0);

        let me = unsafe { Self::new_unchecked(layouts, buffer, capacity, 0) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<Ptrs<'_, T>, DowncastError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self {
            ref layouts,
            buffer,
            capacity,
            offset,
        } = self;

        let actual = layouts.field_layouts();
        let expected = context.field_layouts();
        if let Err(error) = check_downcast(actual, expected, capacity) {
            return Err(DowncastError::new(self, error));
        }

        let ptrs = unsafe { context.ptrs_from_buffer(buffer.cast(), capacity) };
        let ptrs = unsafe { context.ptrs_add(ptrs, offset) };
        Ok(ptrs)
    }
}

impl<D, P> ErasedSoaPtrs<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { layouts, .. } = self;
        layouts
    }
}

impl<'a, D, P> ErasedSoaPtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'e, E>(&'a self, origin: &'e ErasedSoaPtrs<E, P>) -> isize
    where
        E: FieldLayouts<'e> + ?Sized,
    {
        let Self {
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        assert_eq!(buffer, origin.buffer);
        assert_eq!(capacity, origin.capacity);
        assert_layouts(layouts.field_layouts(), origin.field_layouts());

        let offset = offset.cast_signed();
        let origin_offset = origin.offset().cast_signed();
        offset.wrapping_sub(origin_offset)
    }
}

impl<'a, D, P> ErasedSoaPtrs<D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaPtrsIter<D::OutputIter, P, BufferOffsetsOf<D::OutputItem>> {
        let Self {
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let layouts = layouts.field_layouts().into_iter();
        let offsets = Default::default();
        unsafe { ErasedSoaPtrsIter::new_unchecked(buffer, capacity, offset, offsets, layouts) }
    }

    #[inline]
    pub(super) unsafe fn nth_field_ptr(
        &'a self,
        offsets: &mut BufferOffsetsOf<D::OutputItem>,
        i: usize,
    ) -> ErasedPtr<P> {
        let Self {
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let mut layouts = layouts.field_layouts().into_iter();
        let desc = unsafe { layouts.nth(i).unwrap_unchecked() };

        let buffer_offset = unsafe { offsets.next(capacity, desc) };
        unsafe { field_ptr_from_buffer_offset(buffer, offset, buffer_offset) }
    }
}

impl<D, P> ErasedSoaPtrs<D, P>
where
    D: FieldLayoutsOwned<OutputItem: BufferOffsetsFromSelf> + Clone,
    P: ConstSliceItemPtr<Item: Clone>,
{
    #[inline]
    pub unsafe fn read<T>(
        &self,
    ) -> Result<ErasedSoa<T, D, P::Ptrs>, FromFieldsLayoutsError<T::Error>>
    where
        T: AlignedStorageFromLayout<Item = P::Item>,
    {
        let fields = self.iter().map(|ptr| unsafe { ptr.as_ref_unchecked() });
        let layouts = self.layouts().clone();
        ErasedSoa::try_from_fields_layouts(fields, layouts)
    }
}

impl<D, P> Debug for ErasedSoaPtrs<D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            buffer,
            capacity,
            offset,
            layouts,
        } = self;

        f.debug_struct("ErasedSoaPtrs")
            .field("buffer", buffer)
            .field("capacity", capacity)
            .field("offset", offset)
            .field("layouts", &layouts)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaPtrs<D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let layouts = layouts.clone();
        unsafe { Self::new_unchecked(layouts, buffer, capacity, offset) }
    }
}

impl<D, P> Copy for ErasedSoaPtrs<D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaPtrs<D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedPtr<P>;
    type IntoIter = ErasedSoaPtrsIter<D::OutputIter, P, BufferOffsetsOf<D::OutputItem>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedSoaPtrs<D, P>
where
    D: IntoIterator<Item: WithLayout + BufferOffsetsFromSelf>,
    P: ConstSliceItemPtr,
{
    type Item = ErasedPtr<P>;
    type IntoIter = ErasedSoaPtrsIter<D::IntoIter, P, BufferOffsetsOf<D::Item>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            layouts,
            buffer,
            capacity,
            offset,
        } = self;

        let layouts = layouts.into_iter();
        let offsets = Default::default();
        unsafe { ErasedSoaPtrsIter::new_unchecked(buffer, capacity, offset, offsets, layouts) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaPtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;
    type OutputIter = D::OutputIter;
    type OutputItem = D::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { layouts, .. } = self;
        layouts.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedSoaPtrs<D, P>
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

pub struct ErasedSoaPtrsIter<D, P, F>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    buffer: *const [P::Item],
    capacity: usize,
    offset: usize,
    offsets: F,
    layouts: D,
}

impl<D, P, F> ErasedSoaPtrsIter<D, P, F>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        buffer: *const [P::Item],
        capacity: usize,
        offset: usize,
        offsets: F,
        layouts: D,
    ) -> Self {
        Self {
            buffer,
            capacity,
            offset,
            offsets,
            layouts,
        }
    }
}

impl<D, P, F> ErasedSoaPtrsIter<D, P, F>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { layouts, .. } = self;
        layouts
    }
}

impl<D, P, F> ErasedSoaPtrsIter<D, P, F>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    pub unsafe fn next_unchecked(&mut self) -> ErasedPtr<P> {
        let Self {
            ref mut offsets,
            ref mut layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let desc = unsafe { layouts.next().unwrap_unchecked() };
        let buffer_offset = unsafe { offsets.next(capacity, desc) };
        unsafe { field_ptr_from_buffer_offset(buffer, offset, buffer_offset) }
    }
}

impl<'a, D, P, F> ErasedSoaPtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaPtrsIter<D::OutputIter, P, F> {
        let Self {
            ref offsets,
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let offsets = offsets.clone();
        let layouts = layouts.field_layouts().into_iter();
        unsafe { ErasedSoaPtrsIter::new_unchecked(buffer, capacity, offset, offsets, layouts) }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedSoaPtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    type Item = ErasedPtr<P>;
    type IntoIter = ErasedSoaPtrsIter<D::OutputIter, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedSoaPtrsIter<D, P, F>
where
    D: FieldLayoutsOwned + ?Sized,
    P: ConstSliceItemPtr + Debug,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<D, P, F> Clone for ErasedSoaPtrsIter<D, P, F>
where
    D: Clone,
    P: ConstSliceItemPtr,
    F: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref offsets,
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let offsets = offsets.clone();
        let layouts = layouts.clone();
        unsafe { Self::new_unchecked(buffer, capacity, offset, offsets, layouts) }
    }
}

impl<D, P, F> Iterator for ErasedSoaPtrsIter<D, P, F>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut offsets,
            ref mut layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let desc = layouts.next()?;
        let buffer_offset = unsafe { offsets.next(capacity, desc) };
        let item = unsafe { field_ptr_from_buffer_offset(buffer, offset, buffer_offset) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { layouts, .. } = self;
        layouts.size_hint()
    }
}

impl<D, P, F> ExactSizeIterator for ErasedSoaPtrsIter<D, P, F>
where
    D: ExactSizeIterator<Item: WithLayout> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { layouts, .. } = self;
        layouts.len()
    }
}

impl<D, P, F> FusedIterator for ErasedSoaPtrsIter<D, P, F>
where
    D: FusedIterator<Item: WithLayout> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedSoaPtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;
    type OutputIter = D::OutputIter;
    type OutputItem = D::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { layouts, .. } = self;
        layouts.field_layouts()
    }
}

impl<D, P, F> CovariantFieldLayouts for ErasedSoaPtrsIter<D, P, F>
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

#[inline]
unsafe fn field_ptr_from_buffer_offset<P, T>(
    buffer: *const [P::Item],
    offset: usize,
    buffer_offset: BufferOffset<T>,
) -> ErasedPtr<P>
where
    P: ConstSliceItemPtr,
    T: WithLayout,
{
    let (index, layout) = {
        let BufferOffset { desc, offset } = buffer_offset;
        (bytes_to_items::<P::Item>(offset), desc.layout())
    };

    let ptr = unsafe { P::from_slice(buffer, index) };
    unsafe { ErasedPtr::from_parts(layout, ptr).add(offset) }
}
