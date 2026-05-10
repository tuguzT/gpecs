use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::NonNull,
};

use crate::{
    CovariantFieldLayouts, ErasedSoaMutPtrs,
    assert::{assert_layouts, check_downcast},
    data::ErasedNonNullPtr,
    error::{DowncastError, InsufficientAlignError},
    layout::{WithLayout, bytes_to_items},
    offsets::{BufferOffsetsFrom, BufferOffsetsFromSelf, BufferOffsetsOf},
    ptr::slice::{NonNullAsPtr, NonNullSliceItemPtr},
    soa::{
        field::{
            BufferOffset, FieldLayouts, FieldLayoutsItem, FieldLayoutsOutput, FieldLayoutsOwned,
        },
        traits::{AllocSoa, AllocSoaContext, NonNullPtrs, RawSoaContext},
    },
};

pub struct ErasedSoaNonNullPtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    buffer: NonNull<[P::Item]>,
    capacity: usize,
    offset: usize,
    layouts: D,
}

impl<D, P> ErasedSoaNonNullPtrs<D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn new(ptrs: ErasedSoaMutPtrs<D, NonNullAsPtr<P>>) -> Option<Self> {
        let (layouts, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = NonNull::new(buffer)?;

        let me = unsafe { Self::from_parts(layouts, buffer, capacity, offset) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptrs: ErasedSoaMutPtrs<D, NonNullAsPtr<P>>) -> Self {
        let (layouts, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = unsafe { NonNull::new_unchecked(buffer) };

        unsafe { Self::from_parts(layouts, buffer, capacity, offset) }
    }

    #[inline]
    pub unsafe fn from_parts(
        layouts: D,
        buffer: NonNull<[P::Item]>,
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
    pub fn into_parts(self) -> (D, NonNull<[P::Item]>, usize, usize) {
        let Self {
            layouts,
            buffer,
            capacity,
            offset,
        } = self;
        (layouts, buffer, capacity, offset)
    }

    #[inline]
    pub unsafe fn map_layouts<N, F>(self, f: F) -> ErasedSoaNonNullPtrs<N, P>
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
        unsafe { ErasedSoaNonNullPtrs::from_parts(layouts, buffer, capacity, offset) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { offset, .. } = self;

        let offset = unsafe { offset.unchecked_add(count) };
        Self { offset, ..self }
    }
}

impl<D, P> ErasedSoaNonNullPtrs<D, P>
where
    D: FieldLayoutsOwned,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn dangling(layouts: D) -> Result<Self, InsufficientAlignError> {
        let ptrs = ErasedSoaMutPtrs::dangling(layouts)?;
        let me = unsafe { Self::new_unchecked(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<NonNullPtrs<'_, T>, DowncastError<Self>>
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

        unsafe {
            let ptrs = context.ptrs_from_buffer_mut(buffer.as_ptr().cast(), capacity);
            let ptrs = context.ptrs_add_mut(ptrs, offset);
            let ptrs = context.ptrs_to_nonnull(ptrs);
            Ok(ptrs)
        }
    }
}

impl<D, P> ErasedSoaNonNullPtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
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

impl<'a, D, P> ErasedSoaNonNullPtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'e, E>(&'a self, origin: &'e ErasedSoaNonNullPtrs<E, P>) -> isize
    where
        E: FieldLayouts<'e> + ?Sized,
    {
        let Self {
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        assert_eq!(buffer, origin.as_buffer());
        assert_eq!(capacity, origin.capacity());
        assert_layouts(layouts.field_layouts(), origin.field_layouts());

        let offset = offset.cast_signed();
        let origin_offset = origin.offset().cast_signed();
        offset.wrapping_sub(origin_offset)
    }
}

impl<'a, D, P> ErasedSoaNonNullPtrs<D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedSoaNonNullPtrsIter<D::OutputIter, P, BufferOffsetsOf<D::OutputItem>> {
        let Self {
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let layouts = layouts.field_layouts().into_iter();
        let from = Default::default();
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(buffer, capacity, offset, from, layouts) }
    }

    #[inline]
    pub(super) unsafe fn nth_field_ptr(
        &'a self,
        offsets: &mut BufferOffsetsOf<D::OutputItem>,
        i: usize,
    ) -> ErasedNonNullPtr<P> {
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

    #[inline]
    #[track_caller]
    pub unsafe fn swap<'e, E>(&'a mut self, with: &'e mut ErasedSoaNonNullPtrs<E, P>)
    where
        E: FieldLayouts<'e, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    {
        let n = assert_layouts(self.field_layouts(), with.field_layouts());

        let this_offsets = &mut Default::default();
        let with_offsets = &mut Default::default();
        for i in 0..n {
            let this = unsafe { self.nth_field_ptr(this_offsets, i) };
            let with = unsafe { with.nth_field_ptr(with_offsets, i) };
            unsafe { this.swap(with) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_forward<'e, E>(
        &'a mut self,
        src: &'e ErasedSoaNonNullPtrs<E, P>,
        count: usize,
    ) where
        E: FieldLayouts<'e, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    {
        let n = assert_layouts(self.field_layouts(), src.field_layouts());

        let dst_offsets = &mut Default::default();
        let src_offsets = &mut Default::default();
        for i in 0..n {
            let dst = unsafe { self.nth_field_ptr(dst_offsets, i) };
            let src = unsafe { src.nth_field_ptr(src_offsets, i) };
            unsafe { dst.copy_from(src, count) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_backward<'e, E>(
        &'a mut self,
        src: &'e ErasedSoaNonNullPtrs<E, P>,
        count: usize,
    ) where
        E: FieldLayouts<'e, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    {
        #[inline]
        fn rec<'dst, 'src, D, E, P>(
            dst_ptrs: &'dst ErasedSoaNonNullPtrs<D, P>,
            dst_offsets: &mut BufferOffsetsOf<D::OutputItem>,
            src_ptrs: &'src ErasedSoaNonNullPtrs<E, P>,
            src_offsets: &mut BufferOffsetsOf<E::OutputItem>,
            i: usize,
            n: usize,
            count: usize,
        ) where
            D: FieldLayouts<'dst, OutputItem: BufferOffsetsFromSelf> + ?Sized,
            E: FieldLayouts<'src, OutputItem: BufferOffsetsFromSelf> + ?Sized,
            P: NonNullSliceItemPtr,
        {
            if i >= n {
                return;
            }

            let dst = unsafe { dst_ptrs.nth_field_ptr(dst_offsets, i) };
            let src = unsafe { src_ptrs.nth_field_ptr(src_offsets, i) };

            let i = i + 1;
            rec(dst_ptrs, dst_offsets, src_ptrs, src_offsets, i, n, count);

            unsafe { dst.copy_from(src, count) }
        }

        let n = assert_layouts(self.field_layouts(), src.field_layouts());

        let dst_offsets = &mut Default::default();
        let src_offsets = &mut Default::default();
        rec(self, dst_offsets, src, src_offsets, 0, n, count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<'e, E>(
        &'a mut self,
        src: &'e ErasedSoaNonNullPtrs<E, P>,
        count: usize,
    ) where
        E: FieldLayouts<'e, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    {
        let n = assert_layouts(self.field_layouts(), src.field_layouts());

        let dst_offsets = &mut Default::default();
        let src_offsets = &mut Default::default();
        for i in 0..n {
            let dst = unsafe { self.nth_field_ptr(dst_offsets, i) };
            let src = unsafe { src.nth_field_ptr(src_offsets, i) };
            unsafe { dst.copy_from_nonoverlapping(src, count) }
        }
    }
}

impl<D, P> Debug for ErasedSoaNonNullPtrs<D, P>
where
    D: Debug + ?Sized,
    P: NonNullSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            buffer,
            capacity,
            offset,
            layouts,
        } = self;

        f.debug_struct("ErasedSoaNonNullPtrs")
            .field("buffer", buffer)
            .field("capacity", capacity)
            .field("offset", offset)
            .field("layouts", &layouts)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaNonNullPtrs<D, P>
where
    D: Clone,
    P: NonNullSliceItemPtr,
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
        unsafe { Self::from_parts(layouts, buffer, capacity, offset) }
    }
}

impl<D, P> Copy for ErasedSoaNonNullPtrs<D, P>
where
    D: Copy,
    P: NonNullSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaNonNullPtrs<D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::OutputIter, P, BufferOffsetsOf<D::OutputItem>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedSoaNonNullPtrs<D, P>
where
    D: IntoIterator<Item: WithLayout + BufferOffsetsFromSelf>,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::IntoIter, P, BufferOffsetsOf<D::Item>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            layouts,
            buffer,
            capacity,
            offset,
        } = self;

        let layouts = layouts.into_iter();
        let from = Default::default();
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(buffer, capacity, offset, from, layouts) }
    }
}

impl<D, P> From<ErasedSoaNonNullPtrs<D, P>> for ErasedSoaMutPtrs<D, NonNullAsPtr<P>>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn from(ptrs: ErasedSoaNonNullPtrs<D, P>) -> Self {
        let (layouts, ptr, capacity, offset) = ptrs.into_parts();
        let ptr = ptr.as_ptr();
        unsafe { ErasedSoaMutPtrs::new_unchecked(layouts, ptr, capacity, offset) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaNonNullPtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
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

impl<D, P> CovariantFieldLayouts for ErasedSoaNonNullPtrs<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    buffer: NonNull<[P::Item]>,
    capacity: usize,
    offset: usize,
    offsets: F,
    layouts: D,
}

impl<D, P, F> ErasedSoaNonNullPtrsIter<D, P, F>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        buffer: NonNull<[P::Item]>,
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

impl<D, P, F> ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
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

impl<D, P, F> ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    pub unsafe fn next_unchecked(&mut self) -> ErasedNonNullPtr<P> {
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

impl<'a, D, P, F> ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaNonNullPtrsIter<D::OutputIter, P, F> {
        let Self {
            ref offsets,
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let offsets = offsets.clone();
        let layouts = layouts.field_layouts().into_iter();
        unsafe {
            ErasedSoaNonNullPtrsIter::new_unchecked(buffer, capacity, offset, offsets, layouts)
        }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    type Item = ErasedNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::OutputIter, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: FieldLayoutsOwned + ?Sized,
    P: NonNullSliceItemPtr + Debug,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<D, P, F> Clone for ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: Clone,
    P: NonNullSliceItemPtr,
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

impl<D, P, F> Iterator for ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedNonNullPtr<P>;

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

impl<D, P, F> ExactSizeIterator for ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: ExactSizeIterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { layouts, .. } = self;
        layouts.len()
    }
}

impl<D, P, F> FusedIterator for ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: FusedIterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
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

impl<D, P, F> CovariantFieldLayouts for ErasedSoaNonNullPtrsIter<D, P, F>
where
    D: CovariantFieldLayouts + ?Sized,
    P: NonNullSliceItemPtr,
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
    buffer: NonNull<[P::Item]>,
    offset: usize,
    buffer_offset: BufferOffset<T>,
) -> ErasedNonNullPtr<P>
where
    P: NonNullSliceItemPtr,
    T: WithLayout,
{
    let (index, layout) = {
        let BufferOffset { desc, offset } = buffer_offset;
        (bytes_to_items::<P::Item>(offset), desc.layout())
    };

    let ptr = unsafe { P::from_slice(buffer, index) };
    unsafe { ErasedNonNullPtr::from_parts(layout, ptr).add(offset) }
}
