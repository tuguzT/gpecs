use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
};

use crate::{
    bytes_to_items::from_bytes_to_items,
    erased::{
        CovariantFieldDescriptors, ErasedSoaMutPtrs,
        assert::{assert_descriptors, check_into_value},
        error::ErasedSoaIntoValueError,
    },
    error::InsufficientAlignError,
    field::ErasedFieldNonNullPtr,
    slice_item_ptr::{NonNullAsPtr, NonNullSliceItemPtr},
    soa::{
        field::{
            BufferOffset, BufferOffsets, FieldDescriptor, FieldDescriptors, FieldDescriptorsIter,
            FieldDescriptorsOwned, buffer_offsets,
        },
        traits::{AllocSoa, AllocSoaContext, NonNullPtrs, RawSoaContext},
    },
};

pub struct ErasedSoaNonNullPtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    phantom: PhantomData<P>,
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
            phantom: PhantomData,
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
            ..
        } = self;
        (descriptors, buffer, capacity, offset)
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
}

impl<D, P> ErasedSoaNonNullPtrs<D, P>
where
    D: FieldDescriptorsOwned,
    P: NonNullSliceItemPtr<Item = MaybeUninit<u8>>,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<NonNullPtrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;

        let result = check_into_value(descriptors.field_descriptors(), context.field_descriptors());
        if let Err(error) = result {
            return Err(ErasedSoaIntoValueError::new(self, error));
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
            ..
        } = *self;

        assert_eq!(buffer, origin.as_buffer());
        assert_eq!(capacity, origin.capacity());
        assert_descriptors(descriptors.field_descriptors(), origin.field_descriptors());

        let offset = offset.cast_signed();
        let origin_offset = origin.offset().cast_signed();
        offset.wrapping_sub(origin_offset)
    }
}

impl<'a, D, P, U> ErasedSoaNonNullPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;

        let descriptors = descriptors.field_descriptors().into_iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P, U> ErasedSoaNonNullPtrs<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    #[track_caller]
    pub unsafe fn swap<'e, E>(&mut self, with: &'e mut ErasedSoaNonNullPtrs<E, P>)
    where
        E: FieldDescriptors<'e> + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), with.field_descriptors());

        for (this, that) in itertools::zip_eq(self.iter(), with.iter()) {
            unsafe { this.swap(that) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from<'e, E>(&mut self, from: &'e ErasedSoaNonNullPtrs<E, P>, count: usize)
    where
        E: FieldDescriptors<'e> + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), from.field_descriptors());

        for (this, from) in itertools::zip_eq(self.iter(), from) {
            unsafe { this.copy_from(from, count) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_rev<'e, E>(
        &mut self,
        from: &'e ErasedSoaNonNullPtrs<E, P>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), from.field_descriptors());

        #[inline]
        #[expect(clippy::items_after_statements)]
        fn rec<I, P, U>(iter: I, count: usize)
        where
            I: IntoIterator<Item = (ErasedFieldNonNullPtr<P>, ErasedFieldNonNullPtr<P>)>,
            P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
        {
            let mut iter = iter.into_iter();
            let Some((to, from)) = iter.next() else {
                return;
            };

            rec(iter, count);
            unsafe { to.copy_from(from, count) }
        }

        rec(itertools::zip_eq(self.iter(), from), count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<'e, E>(
        &mut self,
        from: &'e ErasedSoaNonNullPtrs<E, P>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), from.field_descriptors());

        for (this, from) in itertools::zip_eq(self.iter(), from) {
            unsafe { this.copy_from_nonoverlapping(from, count) }
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
            ..
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
            ..
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

impl<'a, D, P, U> IntoIterator for &'a ErasedSoaNonNullPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedFieldNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, U> IntoIterator for ErasedSoaNonNullPtrs<D, P>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedFieldNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;

        let descriptors = descriptors.into_iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
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
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedSoaNonNullPtrsIter<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    phantom: PhantomData<P>,
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
            phantom: PhantomData,
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
}

impl<D, P, U> ErasedSoaNonNullPtrsIter<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    unsafe fn field_ptr_from_buffer_offset(
        &self,
        offset: BufferOffset,
    ) -> ErasedFieldNonNullPtr<P> {
        let BufferOffset { desc, offset, .. } = offset;
        let index = from_bytes_to_items::<U>(offset);

        let Self { buffer, offset, .. } = *self;
        let ptr = unsafe { P::from_slice(buffer, index) };
        unsafe { ErasedFieldNonNullPtr::from_parts(desc, ptr).add(offset) }
    }
}

impl<'a, D, P, U> ErasedSoaNonNullPtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self {
            ref inner,
            buffer,
            offset,
            ..
        } = *self;

        let layout = inner.layout();
        let capacity = inner.capacity();
        let fields = inner.as_inner().field_descriptors().into_iter();

        let inner = unsafe { BufferOffsets::from_parts(layout, capacity, fields) };
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P, U> Debug for ErasedSoaNonNullPtrsIter<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>> + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
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
            ..
        } = *self;

        let inner = inner.clone();
        unsafe { Self::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P, U> Iterator for ErasedSoaNonNullPtrsIter<D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedFieldNonNullPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;

        let offset = inner.next()?;
        let offset = unsafe { offset.unwrap_unchecked() };
        let item = unsafe { self.field_ptr_from_buffer_offset(offset) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<D, P, U> ExactSizeIterator for ErasedSoaNonNullPtrsIter<D, P>
where
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<D, P, U> FusedIterator for ErasedSoaNonNullPtrsIter<D, P>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
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
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}
