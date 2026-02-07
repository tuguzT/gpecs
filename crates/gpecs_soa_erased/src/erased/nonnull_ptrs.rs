use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
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
    slice_item_ptr::NonNullSliceItemPtr,
    soa::{
        field::{
            BufferOffset, BufferOffsets, FieldDescriptor, FieldDescriptors, FieldDescriptorsIter,
            FieldDescriptorsOwned, buffer_offsets,
        },
        traits::{AllocSoa, AllocSoaContext, NonNullPtrs, RawSoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<fn() -> P>,
    buffer: NonNull<[MaybeUninit<A>]>,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D, P, A> ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked<E>(ptrs: ErasedSoaMutPtrs<D, E, A>) -> Self {
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
            phantom: PhantomData,
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

impl<D, P, A> ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub fn new<E>(ptrs: ErasedSoaMutPtrs<D, E, A>) -> Option<Self> {
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = NonNull::new(buffer)?;

        let me = unsafe { Self::from_parts(descriptors, buffer, capacity, offset) };
        Some(me)
    }

    #[inline]
    pub fn dangling(descriptors: D) -> Result<Self, InsufficientAlignError> {
        let ptrs = ErasedSoaMutPtrs::<_, P, _>::dangling(descriptors)?;
        let me = unsafe { Self::new_unchecked(ptrs) };
        Ok(me)
    }
}

impl<D, P> ErasedSoaNonNullPtrs<D, P, u8>
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

impl<D, P, A> ErasedSoaNonNullPtrs<D, P, A>
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

impl<'a, D, P, A> ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'e, E>(&'a self, origin: &'e ErasedSoaNonNullPtrs<E, P, A>) -> isize
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

impl<'a, D, P, A> ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
    P: NonNullSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P, A> {
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

impl<D, P, A> ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
    P: NonNullSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptorsOwned + ?Sized,
{
    #[inline]
    #[track_caller]
    pub unsafe fn swap<'e, E>(&mut self, with: &'e mut ErasedSoaNonNullPtrs<E, P, A>)
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
    pub unsafe fn copy_from<'e, E>(&mut self, from: &'e ErasedSoaNonNullPtrs<E, P, A>, count: usize)
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
        from: &'e ErasedSoaNonNullPtrs<E, P, A>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), from.field_descriptors());

        #[inline]
        #[expect(clippy::items_after_statements)]
        fn rec<A, P, I>(iter: I, count: usize)
        where
            A: AddressableUnit,
            P: NonNullSliceItemPtr<Item = MaybeUninit<A>>,
            I: IntoIterator<Item = (ErasedFieldNonNullPtr<P>, ErasedFieldNonNullPtr<P>)>,
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
        from: &'e ErasedSoaNonNullPtrs<E, P, A>,
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

impl<D, P, A> Debug for ErasedSoaNonNullPtrs<D, P, A>
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

impl<D, P, A> Clone for ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
    D: Clone,
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

impl<D, P, A> Copy for ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<'a, D, P, A> IntoIterator for &'a ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
    P: NonNullSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Item = ErasedFieldNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, A> IntoIterator for ErasedSoaNonNullPtrs<D, P, A>
where
    A: AddressableUnit,
    P: NonNullSliceItemPtr<Item = MaybeUninit<A>>,
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    type Item = ErasedFieldNonNullPtr<P>;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::IntoIter, P, A>;

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

impl<'a, D, P, A> FieldDescriptors<'a> for ErasedSoaNonNullPtrs<D, P, A>
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

impl<D, P, A> CovariantFieldDescriptors for ErasedSoaNonNullPtrs<D, P, A>
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

pub struct ErasedSoaNonNullPtrsIter<D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<fn() -> P>,
    buffer: NonNull<[MaybeUninit<A>]>,
    offset: usize,
    inner: BufferOffsets<D>,
}

impl<D, P, A> ErasedSoaNonNullPtrsIter<D, P, A>
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
            phantom: PhantomData,
            buffer,
            offset,
            inner,
        }
    }
}

impl<D, P, A> ErasedSoaNonNullPtrsIter<D, P, A>
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

impl<'a, D, P, A> ErasedSoaNonNullPtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: NonNullSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaNonNullPtrsIter<FieldDescriptorsIter<'a, D>, P, A> {
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

impl<D, P, A> Debug for ErasedSoaNonNullPtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: NonNullSliceItemPtr<Item = MaybeUninit<A>> + Debug,
    D: FieldDescriptorsOwned + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, P, A> Clone for ErasedSoaNonNullPtrsIter<D, P, A>
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
            ..
        } = *self;

        let inner = inner.clone();
        unsafe { Self::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P, A> Iterator for ErasedSoaNonNullPtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: NonNullSliceItemPtr<Item = MaybeUninit<A>>,
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    type Item = ErasedFieldNonNullPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut inner,
            buffer,
            offset,
            ..
        } = *self;

        let field_ptr = {
            let BufferOffset { desc, offset, .. } = unsafe { inner.next()?.unwrap_unchecked() };
            let index = offset.div_ceil(size_of::<A>());
            let ptr = unsafe { P::from_slice(buffer, index) };
            unsafe { ErasedFieldNonNullPtr::from_parts(desc, ptr) }
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

impl<D, P, A> ExactSizeIterator for ErasedSoaNonNullPtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: NonNullSliceItemPtr<Item = MaybeUninit<A>>,
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<D, P, A> FusedIterator for ErasedSoaNonNullPtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: NonNullSliceItemPtr<Item = MaybeUninit<A>>,
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
}

impl<'a, D, P, A> FieldDescriptors<'a> for ErasedSoaNonNullPtrsIter<D, P, A>
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

impl<D, P, A> CovariantFieldDescriptors for ErasedSoaNonNullPtrsIter<D, P, A>
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
