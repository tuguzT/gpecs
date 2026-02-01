use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::MaybeUninit,
    slice,
};

use crate::{
    erased::{
        CovariantFieldDescriptors, ErasedSoaPtrs, ErasedSoaPtrsIter,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError},
    },
    field::ErasedFieldRef,
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOwned},
        traits::{AllocSoa, Refs, Soa, SoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaRefs<'a, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a [MaybeUninit<A>]>,
    ptrs: ErasedSoaPtrs<D, A>,
}

impl<'a, D, A> ErasedSoaRefs<'a, D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: &'a [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaPtrs::new_unchecked(descriptors, buffer, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaPtrs<D, A>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a [MaybeUninit<A>], usize, usize) {
        let Self { ptrs, .. } = self;
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();

        let buffer = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, A> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<'a, D, A> ErasedSoaRefs<'a, D, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: &'a [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, ErasedSoaPtrsError> {
        let ptrs = ErasedSoaPtrs::new(descriptors, buffer, capacity, offset)?;
        let me = unsafe { Self::from_ptrs(ptrs) };
        Ok(me)
    }
}

impl<'a, D> ErasedSoaRefs<'a, D, u8>
where
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<Refs<'_, 'a, T>, ErasedSoaIntoValueError<Self>>
    where
        T: AllocSoa + Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let refs = unsafe { context.ptrs_to_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, A> ErasedSoaRefs<'_, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<A>] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
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
}

impl<'a, D, A> ErasedSoaRefs<'_, D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<D, A> Debug for ErasedSoaRefs<'_, D, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_struct("ErasedSoaRefs")
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<D, A> Clone for ErasedSoaRefs<'_, D, A>
where
    A: AddressableUnit,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { phantom, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { phantom, ptrs }
    }
}

impl<D, A> Copy for ErasedSoaRefs<'_, D, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<'a, D, A> IntoIterator for &'a ErasedSoaRefs<'_, D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Item = ErasedFieldRef<'a, A>;
    type IntoIter = ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, A> IntoIterator for ErasedSoaRefs<'a, D, A>
where
    A: AddressableUnit,
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    type Item = ErasedFieldRef<'a, A>;
    type IntoIter = ErasedSoaRefsIter<'a, D::IntoIter, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, A> FieldDescriptors<'a> for ErasedSoaRefs<'_, D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }
}

impl<D, A> CovariantFieldDescriptors for ErasedSoaRefs<'_, D, A>
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

pub struct ErasedSoaRefsIter<'a, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a [MaybeUninit<A>]>,
    ptrs: ErasedSoaPtrsIter<D, A>,
}

impl<D, A> ErasedSoaRefsIter<'_, D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn from_ptrs(ptrs: ErasedSoaPtrsIter<D, A>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, A> ErasedSoaRefsIter<'_, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<A>] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
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
}

impl<'a, D, A> ErasedSoaRefsIter<'_, D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.entries();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<D, A> Debug for ErasedSoaRefsIter<'_, D, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, A> Clone for ErasedSoaRefsIter<'_, D, A>
where
    A: AddressableUnit,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { phantom, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { phantom, ptrs }
    }
}

impl<'a, D, A> Iterator for ErasedSoaRefsIter<'a, D, A>
where
    A: AddressableUnit,
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    type Item = ErasedFieldRef<'a, A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ptrs, .. } = self;

        let item = unsafe { ptrs.next()?.deref() };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, A> ExactSizeIterator for ErasedSoaRefsIter<'_, D, A>
where
    A: AddressableUnit,
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, A> FusedIterator for ErasedSoaRefsIter<'_, D, A>
where
    A: AddressableUnit,
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
}

impl<'a, D, A> FieldDescriptors<'a> for ErasedSoaRefsIter<'_, D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }
}

impl<D, A> CovariantFieldDescriptors for ErasedSoaRefsIter<'_, D, A>
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
