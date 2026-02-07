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
    slice_item_ptr::ConstSliceItemPtr,
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOwned},
        traits::{AllocSoa, Refs, Soa, SoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaRefs<'a, D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a [MaybeUninit<A>]>,
    ptrs: ErasedSoaPtrs<D, P, A>,
}

impl<'a, D, P, A> ErasedSoaRefs<'a, D, P, A>
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
    pub unsafe fn from_ptrs(ptrs: ErasedSoaPtrs<D, P, A>) -> Self {
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
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, P, A> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<'a, D, P, A> ErasedSoaRefs<'a, D, P, A>
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

impl<'a, D, P> ErasedSoaRefs<'a, D, P, u8>
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

impl<D, P, A> ErasedSoaRefs<'_, D, P, A>
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

impl<'a, D, P, A> ErasedSoaRefs<'_, D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, P, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<D, P, A> Debug for ErasedSoaRefs<'_, D, P, A>
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

impl<D, P, A> Clone for ErasedSoaRefs<'_, D, P, A>
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

impl<D, P, A> Copy for ErasedSoaRefs<'_, D, P, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<'a, D, P, A> IntoIterator for &'a ErasedSoaRefs<'_, D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Item = ErasedFieldRef<'a, P>;
    type IntoIter = ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, P, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P, A> IntoIterator for ErasedSoaRefs<'a, D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    type Item = ErasedFieldRef<'a, P>;
    type IntoIter = ErasedSoaRefsIter<'a, D::IntoIter, P, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P, A> FieldDescriptors<'a> for ErasedSoaRefs<'_, D, P, A>
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

impl<D, P, A> CovariantFieldDescriptors for ErasedSoaRefs<'_, D, P, A>
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

pub struct ErasedSoaRefsIter<'a, D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a [MaybeUninit<A>]>,
    ptrs: ErasedSoaPtrsIter<D, P, A>,
}

impl<D, P, A> ErasedSoaRefsIter<'_, D, P, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn from_ptrs(ptrs: ErasedSoaPtrsIter<D, P, A>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, P, A> ErasedSoaRefsIter<'_, D, P, A>
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

impl<'a, D, P, A> ErasedSoaRefsIter<'_, D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, P, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.entries();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<D, P, A> Debug for ErasedSoaRefsIter<'_, D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptorsOwned + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, P, A> Clone for ErasedSoaRefsIter<'_, D, P, A>
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

impl<'a, D, P, A> Iterator for ErasedSoaRefsIter<'a, D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    type Item = ErasedFieldRef<'a, P>;

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

impl<D, P, A> ExactSizeIterator for ErasedSoaRefsIter<'_, D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P, A> FusedIterator for ErasedSoaRefsIter<'_, D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
}

impl<'a, D, P, A> FieldDescriptors<'a> for ErasedSoaRefsIter<'_, D, P, A>
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

impl<D, P, A> CovariantFieldDescriptors for ErasedSoaRefsIter<'_, D, P, A>
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
