use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::NonNull,
};

use crate::{
    CovariantFieldDescriptors, ErasedSoaMutPtrs,
    assert::{assert_descriptors, check_downcast},
    data::ErasedNonNullPtr,
    error::DowncastError,
    error::InsufficientAlignError,
    layout::bytes_to_items,
    ptr::slice::{NonNullAsPtr, NonNullSliceItemPtr},
    soa::{
        field::{
            BufferOffset, BufferOffsets, FieldDescriptor, FieldDescriptors, FieldDescriptorsIter,
            FieldDescriptorsOutput, FieldDescriptorsOwned, IntoCopiedFieldDescriptors,
            RawBufferOffsets, buffer_offsets,
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
    descriptors: D,
}

impl<D, P> ErasedSoaNonNullPtrs<D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn new(ptrs: ErasedSoaMutPtrs<D, NonNullAsPtr<P>>) -> Option<Self> {
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = NonNull::new(buffer)?;

        let me = unsafe { Self::from_parts(descriptors, buffer, capacity, offset) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptrs: ErasedSoaMutPtrs<D, NonNullAsPtr<P>>) -> Self {
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = unsafe { NonNull::new_unchecked(buffer) };

        unsafe { Self::from_parts(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    pub unsafe fn from_parts(
        descriptors: D,
        buffer: NonNull<[P::Item]>,
        capacity: usize,
        offset: usize,
    ) -> Self {
        Self {
            buffer,
            capacity,
            offset,
            descriptors,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, NonNull<[P::Item]>, usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub unsafe fn map_descriptors<N, F>(self, f: F) -> ErasedSoaNonNullPtrs<N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        let descriptors = f(descriptors);
        unsafe { ErasedSoaNonNullPtrs::from_parts(descriptors, buffer, capacity, offset) }
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
    D: FieldDescriptorsOwned,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn dangling(descriptors: D) -> Result<Self, InsufficientAlignError> {
        let ptrs = ErasedSoaMutPtrs::dangling(descriptors)?;
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
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        let actual = descriptors.field_descriptors();
        let expected = context.field_descriptors();
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
    pub fn descriptors(&self) -> &D {
        let Self { descriptors, .. } = self;
        descriptors
    }

    #[inline]
    fn raw_buffer_offsets(&self) -> RawBufferOffsets {
        let Self { capacity, .. } = *self;
        RawBufferOffsets::new(capacity)
    }
}

impl<'a, D, P> ErasedSoaNonNullPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'e, E>(&'a self, origin: &'e ErasedSoaNonNullPtrs<E, P>) -> isize
    where
        E: FieldDescriptors<'e> + ?Sized,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        assert_eq!(buffer, origin.as_buffer());
        assert_eq!(capacity, origin.capacity());
        assert_descriptors(descriptors.field_descriptors(), origin.field_descriptors());

        let offset = offset.cast_signed();
        let origin_offset = origin.offset().cast_signed();
        offset.wrapping_sub(origin_offset)
    }

    #[inline]
    pub fn iter(&'a self) -> ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.field_descriptors().into_iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
    }

    #[inline]
    pub(super) unsafe fn nth_field_ptr(
        &'a self,
        state: &mut RawBufferOffsets,
        i: usize,
    ) -> ErasedNonNullPtr<P> {
        let Self {
            ref descriptors,
            buffer,
            offset,
            ..
        } = *self;

        let mut descriptors = descriptors.field_descriptors().copied_field_descriptors();
        let desc = unsafe { descriptors.nth(i).unwrap_unchecked() };

        let buffer_offset = BufferOffset {
            offset: unsafe { state.next_unchecked(desc) },
            desc,
        };
        unsafe { field_ptr_from_buffer_offset(buffer, offset, buffer_offset) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<'e, E>(&'a mut self, with: &'e mut ErasedSoaNonNullPtrs<E, P>)
    where
        E: FieldDescriptors<'e> + ?Sized,
    {
        let n = assert_descriptors(self.field_descriptors(), with.field_descriptors());

        let this_state = &mut self.raw_buffer_offsets();
        let with_state = &mut with.raw_buffer_offsets();
        for i in 0..n {
            let this = unsafe { self.nth_field_ptr(this_state, i) };
            let with = unsafe { with.nth_field_ptr(with_state, i) };
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
        E: FieldDescriptors<'e> + ?Sized,
    {
        let n = assert_descriptors(self.field_descriptors(), src.field_descriptors());

        let dst_state = &mut self.raw_buffer_offsets();
        let src_state = &mut src.raw_buffer_offsets();
        for i in 0..n {
            let dst = unsafe { self.nth_field_ptr(dst_state, i) };
            let src = unsafe { src.nth_field_ptr(src_state, i) };
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
        E: FieldDescriptors<'e> + ?Sized,
    {
        #[inline]
        fn rec<'dst, 'src, D, E, P>(
            dst_ptrs: &'dst ErasedSoaNonNullPtrs<D, P>,
            dst_state: &mut RawBufferOffsets,
            src_ptrs: &'src ErasedSoaNonNullPtrs<E, P>,
            src_state: &mut RawBufferOffsets,
            i: usize,
            n: usize,
            count: usize,
        ) where
            D: FieldDescriptors<'dst> + ?Sized,
            E: FieldDescriptors<'src> + ?Sized,
            P: NonNullSliceItemPtr,
        {
            if i >= n {
                return;
            }

            let dst = unsafe { dst_ptrs.nth_field_ptr(dst_state, i) };
            let src = unsafe { src_ptrs.nth_field_ptr(src_state, i) };

            rec(dst_ptrs, dst_state, src_ptrs, src_state, i + 1, n, count);

            unsafe { dst.copy_from(src, count) }
        }

        let n = assert_descriptors(self.field_descriptors(), src.field_descriptors());

        let dst_state = &mut self.raw_buffer_offsets();
        let src_state = &mut src.raw_buffer_offsets();
        rec(self, dst_state, src, src_state, 0, n, count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<'e, E>(
        &'a mut self,
        src: &'e ErasedSoaNonNullPtrs<E, P>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        let n = assert_descriptors(self.field_descriptors(), src.field_descriptors());

        let dst_state = &mut self.raw_buffer_offsets();
        let src_state = &mut src.raw_buffer_offsets();
        for i in 0..n {
            let dst = unsafe { self.nth_field_ptr(dst_state, i) };
            let src = unsafe { src.nth_field_ptr(src_state, i) };
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
            descriptors,
        } = self;

        f.debug_struct("ErasedSoaNonNullPtrs")
            .field("buffer", buffer)
            .field("capacity", capacity)
            .field("offset", offset)
            .field("descriptors", &descriptors)
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
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.clone();
        unsafe { Self::from_parts(descriptors, buffer, capacity, offset) }
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
    D: FieldDescriptors<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedSoaNonNullPtrs<D, P>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        let descriptors = descriptors.into_iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P> From<ErasedSoaNonNullPtrs<D, P>> for ErasedSoaMutPtrs<D, NonNullAsPtr<P>>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn from(ptrs: ErasedSoaNonNullPtrs<D, P>) -> Self {
        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        let ptr = ptr.as_ptr();
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaNonNullPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { descriptors, .. } = self;
        descriptors.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaNonNullPtrs<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
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
    pub fn descriptors(&self) -> &D {
        let Self { inner, .. } = self;
        inner.as_inner()
    }
}

impl<D, P> ErasedSoaNonNullPtrsIter<D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
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
    D: FieldDescriptors<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self {
            ref inner,
            buffer,
            offset,
        } = *self;

        let state = inner.state();
        let fields = inner.as_inner().field_descriptors().into_iter();

        let inner = unsafe { BufferOffsets::from_parts(state, fields) };
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaNonNullPtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedSoaNonNullPtrsIter<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
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
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
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
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
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
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: NonNullSliceItemPtr,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaNonNullPtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner, .. } = self;
        inner.as_inner().field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaNonNullPtrsIter<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}

#[inline]
unsafe fn field_ptr_from_buffer_offset<P>(
    buffer: NonNull<[P::Item]>,
    offset: usize,
    buffer_offset: BufferOffset,
) -> ErasedNonNullPtr<P>
where
    P: NonNullSliceItemPtr,
{
    let (index, layout) = {
        let BufferOffset { desc, offset } = buffer_offset;
        (bytes_to_items::<P::Item>(offset), desc.into())
    };

    let ptr = unsafe { P::from_slice(buffer, index) };
    unsafe { ErasedNonNullPtr::from_parts(layout, ptr).add(offset) }
}
