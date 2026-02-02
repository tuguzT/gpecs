use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
    ptr::NonNull,
};

use crate::{
    erased::{
        CovariantFieldDescriptors, ErasedSoaMutPtrs,
        assert::{assert_descriptors, check_into_value},
        error::ErasedSoaIntoValueError,
    },
    error::InsufficientAlignError,
    field::ErasedFieldNonNullPtr,
    soa::{
        field::{
            BufferOffset, BufferOffsets, FieldDescriptor, FieldDescriptors, FieldDescriptorsIter,
            FieldDescriptorsOwned, buffer_offsets,
        },
        traits::{AllocSoa, AllocSoaContext, NonNullPtrs, RawSoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    buffer: NonNull<[MaybeUninit<A>]>,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D, A> ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(ptrs: ErasedSoaMutPtrs<D, A>) -> Self {
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = unsafe { NonNull::new_unchecked(buffer) };

        unsafe { Self::from_parts(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    pub unsafe fn from_parts(
        descriptors: D,
        buffer: NonNull<[MaybeUninit<A>]>,
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
    pub fn into_parts(self) -> (D, NonNull<[MaybeUninit<A>]>, usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
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

impl<D, A> ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub fn new(ptrs: ErasedSoaMutPtrs<D, A>) -> Option<Self> {
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = NonNull::new(buffer)?;

        let me = unsafe { Self::from_parts(descriptors, buffer, capacity, offset) };
        Some(me)
    }

    #[inline]
    pub fn dangling(descriptors: D) -> Result<Self, InsufficientAlignError> {
        let ptrs = ErasedSoaMutPtrs::dangling(descriptors)?;
        let me = unsafe { Self::new_unchecked(ptrs) };
        Ok(me)
    }
}

impl<D> ErasedSoaNonNullPtrs<D, u8>
where
    D: FieldDescriptorsOwned,
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

impl<D, A> ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[MaybeUninit<A>]> {
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

impl<'a, D, A> ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'e, E>(&'a self, origin: &'e ErasedSoaNonNullPtrs<E, A>) -> isize
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
    pub fn iter(&'a self) -> ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, A> {
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
}

impl<D, A> ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned + ?Sized,
{
    #[inline]
    #[track_caller]
    pub unsafe fn swap<'e, E>(&mut self, with: &'e mut ErasedSoaNonNullPtrs<E, A>)
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
    pub unsafe fn copy_from<'e, E>(&mut self, from: &'e ErasedSoaNonNullPtrs<E, A>, count: usize)
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
        from: &'e ErasedSoaNonNullPtrs<E, A>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), from.field_descriptors());

        #[inline]
        #[expect(clippy::items_after_statements)]
        fn rec<A, I>(iter: I, count: usize)
        where
            A: AddressableUnit,
            I: IntoIterator<Item = (ErasedFieldNonNullPtr<A>, ErasedFieldNonNullPtr<A>)>,
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
        from: &'e ErasedSoaNonNullPtrs<E, A>,
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

impl<D, A> Debug for ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
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

impl<D, A> Clone for ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            buffer,
            capacity,
            offset,
            ref descriptors,
        } = *self;

        let descriptors = descriptors.clone();
        unsafe { Self::from_parts(descriptors, buffer, capacity, offset) }
    }
}

impl<D, A> Copy for ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<'a, D, A> IntoIterator for &'a ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Item = ErasedFieldNonNullPtr<A>;
    type IntoIter = ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, A> IntoIterator for ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    type Item = ErasedFieldNonNullPtr<A>;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::IntoIter, A>;

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

impl<'a, D, A> FieldDescriptors<'a> for ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { descriptors, .. } = self;
        descriptors.field_descriptors()
    }
}

impl<D, A> CovariantFieldDescriptors for ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    buffer: NonNull<[MaybeUninit<A>]>,
    offset: usize,
    inner: BufferOffsets<D>,
}

impl<D, A> ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        inner: BufferOffsets<D>,
        buffer: NonNull<[MaybeUninit<A>]>,
        offset: usize,
    ) -> Self {
        Self {
            buffer,
            offset,
            inner,
        }
    }
}

impl<D, A> ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[MaybeUninit<A>]> {
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

impl<'a, D, A> ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, A> {
        let Self {
            ref inner,
            buffer,
            offset,
        } = *self;

        let layout = inner.layout();
        let capacity = inner.capacity();
        let fields = inner.as_inner().field_descriptors().into_iter();

        let inner = unsafe { BufferOffsets::from_parts(layout, capacity, fields) };
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, A> Debug for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, A> Clone for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: Clone,
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

impl<D, A> Iterator for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    type Item = ErasedFieldNonNullPtr<A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut inner,
            buffer,
            offset,
        } = *self;

        let field_ptr = {
            let BufferOffset { desc, offset, .. } = unsafe { inner.next()?.unwrap_unchecked() };
            unsafe { ErasedFieldNonNullPtr::from_parts(desc, buffer, offset) }
        };

        let item = unsafe { field_ptr.add(offset) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<D, A> ExactSizeIterator for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<D, A> FusedIterator for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
}

impl<'a, D, A> FieldDescriptors<'a> for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner, .. } = self;
        inner.as_inner().field_descriptors()
    }
}

impl<D, A> CovariantFieldDescriptors for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}
