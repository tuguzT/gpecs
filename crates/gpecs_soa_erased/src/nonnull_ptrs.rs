use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    num::NonZeroUsize,
    ptr::NonNull,
};

use crate::{
    CovariantFieldLayouts, ErasedSoaMutPtrs,
    assert::{assert_layouts, check_downcast},
    data::ErasedNonNullPtr,
    error::{DowncastError, InsufficientAlignError},
    layout::bytes_to_items,
    offsets::FieldOffsets,
    ptr::slice::{NonNullAsPtr, NonNullSliceItemPtr},
    soa::{
        field::{
            BufferLayout, BufferOffset, BufferOffsets, FieldLayouts, FieldLayoutsItem,
            FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned, buffer_offsets,
        },
        layout::WithLayout,
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

    #[inline]
    pub fn iter(&'a self) -> ErasedSoaNonNullPtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self {
            ref layouts,
            buffer,
            capacity,
            offset,
        } = *self;

        let layouts = layouts.field_layouts().into_iter();
        let inner = buffer_offsets(layouts, capacity);
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
    }

    #[inline]
    pub(super) unsafe fn nth_field_ptr<F>(
        &'a self,
        offsets: &mut F,
        i: usize,
    ) -> ErasedNonNullPtr<P>
    where
        F: for<'b> FieldOffsets<&'b FieldLayoutsItem<'a, D>>,
    {
        let Self {
            ref layouts,
            buffer,
            offset,
            ..
        } = *self;

        let mut layouts = layouts.field_layouts().into_iter();
        let desc = unsafe { layouts.nth(i).unwrap_unchecked() };

        let buffer_offset = unsafe { offsets.next(&desc) };
        let buffer_offset = BufferOffset::new(desc, buffer_offset);
        unsafe { field_ptr_from_buffer_offset(buffer, offset, buffer_offset) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<'e, E>(&'a mut self, with: &'e mut ErasedSoaNonNullPtrs<E, P>)
    where
        E: FieldLayouts<'e> + ?Sized,
    {
        let n = assert_layouts(self.field_layouts(), with.field_layouts());

        let this_layout = &mut BufferLayout::new(self.capacity(), NonZeroUsize::MIN);
        let with_layout = &mut BufferLayout::new(with.capacity(), NonZeroUsize::MIN);
        for i in 0..n {
            let this = unsafe { self.nth_field_ptr(this_layout, i) };
            let with = unsafe { with.nth_field_ptr(with_layout, i) };
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
        E: FieldLayouts<'e> + ?Sized,
    {
        let n = assert_layouts(self.field_layouts(), src.field_layouts());

        let dst_layout = &mut BufferLayout::new(self.capacity(), NonZeroUsize::MIN);
        let src_layout = &mut BufferLayout::new(src.capacity(), NonZeroUsize::MIN);
        for i in 0..n {
            let dst = unsafe { self.nth_field_ptr(dst_layout, i) };
            let src = unsafe { src.nth_field_ptr(src_layout, i) };
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
        E: FieldLayouts<'e> + ?Sized,
    {
        #[inline]
        fn rec<'dst, 'src, D, E, P>(
            dst_ptrs: &'dst ErasedSoaNonNullPtrs<D, P>,
            dst_layout: &mut BufferLayout,
            src_ptrs: &'src ErasedSoaNonNullPtrs<E, P>,
            src_layout: &mut BufferLayout,
            i: usize,
            n: usize,
            count: usize,
        ) where
            D: FieldLayouts<'dst> + ?Sized,
            E: FieldLayouts<'src> + ?Sized,
            P: NonNullSliceItemPtr,
        {
            if i >= n {
                return;
            }

            let dst = unsafe { dst_ptrs.nth_field_ptr(dst_layout, i) };
            let src = unsafe { src_ptrs.nth_field_ptr(src_layout, i) };

            rec(dst_ptrs, dst_layout, src_ptrs, src_layout, i + 1, n, count);

            unsafe { dst.copy_from(src, count) }
        }

        let n = assert_layouts(self.field_layouts(), src.field_layouts());

        let dst_layout = &mut BufferLayout::new(self.capacity(), NonZeroUsize::MIN);
        let src_layout = &mut BufferLayout::new(src.capacity(), NonZeroUsize::MIN);
        rec(self, dst_layout, src, src_layout, 0, n, count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<'e, E>(
        &'a mut self,
        src: &'e ErasedSoaNonNullPtrs<E, P>,
        count: usize,
    ) where
        E: FieldLayouts<'e> + ?Sized,
    {
        let n = assert_layouts(self.field_layouts(), src.field_layouts());

        let dst_layout = &mut BufferLayout::new(self.capacity(), NonZeroUsize::MIN);
        let src_layout = &mut BufferLayout::new(src.capacity(), NonZeroUsize::MIN);
        for i in 0..n {
            let dst = unsafe { self.nth_field_ptr(dst_layout, i) };
            let src = unsafe { src.nth_field_ptr(src_layout, i) };
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
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedSoaNonNullPtrs<D, P>
where
    D: IntoIterator<Item: WithLayout>,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            layouts,
            buffer,
            capacity,
            offset,
        } = self;

        let layouts = layouts.into_iter();
        let inner = buffer_offsets(layouts, capacity);
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
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

pub struct ErasedSoaNonNullPtrsIter<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    buffer: NonNull<[P::Item]>,
    offset: usize,
    inner: BufferOffsets<D>,
}

impl<D, P> ErasedSoaNonNullPtrsIter<D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        inner: BufferOffsets<D>,
        buffer: NonNull<[P::Item]>,
        offset: usize,
    ) -> Self {
        Self {
            buffer,
            offset,
            inner,
        }
    }
}

impl<D, P> ErasedSoaNonNullPtrsIter<D, P>
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
        let Self { inner, .. } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { inner, .. } = self;
        inner.as_inner()
    }
}

impl<D, P> ErasedSoaNonNullPtrsIter<D, P>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn next_unchecked(&mut self) -> ErasedNonNullPtr<P> {
        let Self {
            ref mut inner,
            buffer,
            offset,
        } = *self;

        let buffer_offset = unsafe { inner.next_unchecked() };
        unsafe { field_ptr_from_buffer_offset(buffer, offset, buffer_offset) }
    }
}

impl<'a, D, P> ErasedSoaNonNullPtrsIter<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaNonNullPtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self {
            ref inner,
            buffer,
            offset,
        } = *self;

        let buffer_layout = inner.buffer();
        let fields = inner.as_inner().field_layouts().into_iter();

        let inner = unsafe { BufferOffsets::from_parts(buffer_layout, fields) };
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaNonNullPtrsIter<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedSoaNonNullPtrsIter<D, P>
where
    D: FieldLayoutsOwned + ?Sized,
    P: NonNullSliceItemPtr + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedSoaNonNullPtrsIter<D, P>
where
    D: Clone,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref inner,
            buffer,
            offset,
        } = *self;

        let inner = inner.clone();
        unsafe { Self::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P> Iterator for ErasedSoaNonNullPtrsIter<D, P>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut inner,
            buffer,
            offset,
        } = *self;

        let buffer_offset = inner.next()?;
        let buffer_offset = unsafe { buffer_offset.unwrap_unchecked() };
        let item = unsafe { field_ptr_from_buffer_offset(buffer, offset, buffer_offset) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedSoaNonNullPtrsIter<D, P>
where
    D: ExactSizeIterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<D, P> FusedIterator for ErasedSoaNonNullPtrsIter<D, P>
where
    D: FusedIterator<Item: WithLayout> + ?Sized,
    P: NonNullSliceItemPtr,
{
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaNonNullPtrsIter<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner, .. } = self;
        inner.as_inner().field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedSoaNonNullPtrsIter<D, P>
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
