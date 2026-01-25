use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaMutPtrs, ErasedSoaMutPtrsIter, ErasedSoaPtrs, ErasedSoaRefsIter,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError},
    },
    field::{ErasedFieldRef, ErasedFieldRefMut},
    soa::{
        field::FieldDescriptor,
        traits::{AllocSoa, RefsMut, Soa, SoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaRefsMut<'a, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [MaybeUninit<A>]>,
    ptrs: ErasedSoaMutPtrs<D, A>,
}

impl<'a, D, A> ErasedSoaRefsMut<'a, D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: &'a mut [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
    ) -> Self {
        let buffer = ptr::from_mut(buffer) as _;
        let ptrs =
            unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, offset) };

        unsafe { Self::from_mut_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(ptrs: ErasedSoaMutPtrs<D, A>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a mut [MaybeUninit<A>], usize, usize) {
        let Self { ptrs, .. } = self;
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();

        let buffer = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, A> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaMutPtrs<D, A> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<'a, D, A> ErasedSoaRefsMut<'a, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: &'a mut [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, ErasedSoaPtrsError> {
        let buffer = ptr::from_mut(buffer) as _;
        let ptrs = ErasedSoaMutPtrs::new(descriptors, buffer, capacity, offset)?;

        let me = unsafe { Self::from_mut_ptrs(ptrs) };
        Ok(me)
    }
}

impl<'a, D> ErasedSoaRefsMut<'a, D, u8>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<RefsMut<'_, 'a, T>, ErasedSoaIntoValueError<Self>>
    where
        T: AllocSoa + Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_mut_ptrs(ptrs) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let refs = unsafe { context.mut_ptrs_to_mut_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, A> ErasedSoaRefsMut<'_, D, A>
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
    pub fn as_mut_buffer(&mut self) -> &mut [MaybeUninit<A>] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
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

impl<D, A> ErasedSoaRefsMut<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaRefsIter<'_, slice::Iter<'_, FieldDescriptor>, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedSoaRefsMutIter<'_, slice::Iter<'_, FieldDescriptor>, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter_mut();
        unsafe { ErasedSoaRefsMutIter::from_ptrs(ptrs) }
    }
}

impl<D, A> Debug for ErasedSoaRefsMut<'_, D, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_struct("ErasedSoaRefsMut")
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<'a, D, A> IntoIterator for &'a ErasedSoaRefsMut<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldRef<'a, A>;
    type IntoIter = ErasedSoaRefsIter<'a, slice::Iter<'a, FieldDescriptor>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, A> IntoIterator for &'a mut ErasedSoaRefsMut<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldRefMut<'a, A>;
    type IntoIter = ErasedSoaRefsMutIter<'a, slice::Iter<'a, FieldDescriptor>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D, A> IntoIterator for ErasedSoaRefsMut<'a, D, A>
where
    A: AddressableUnit,
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldRefMut<'a, A>;
    type IntoIter = ErasedSoaRefsMutIter<'a, D::IntoIter, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaRefsMutIter::from_ptrs(ptrs) }
    }
}

pub struct ErasedSoaRefsMutIter<'a, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [MaybeUninit<A>]>,
    ptrs: ErasedSoaMutPtrsIter<D, A>,
}

impl<D, A> ErasedSoaRefsMutIter<'_, D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn from_ptrs(ptrs: ErasedSoaMutPtrsIter<D, A>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, A> ErasedSoaRefsMutIter<'_, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
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

impl<D, A> ErasedSoaRefsMutIter<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }

    #[inline]
    pub(super) fn debug_entries(
        &self,
    ) -> ErasedSoaRefsMutIter<'_, slice::Iter<'_, FieldDescriptor>, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.debug_entries();
        unsafe { ErasedSoaRefsMutIter::from_ptrs(ptrs) }
    }
}

impl<D, A> Debug for ErasedSoaRefsMutIter<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.debug_entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D, A> Iterator for ErasedSoaRefsMutIter<'a, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldRefMut<'a, A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ptrs, .. } = self;

        let item = unsafe { ptrs.next()?.deref_mut() };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, A> ExactSizeIterator for ErasedSoaRefsMutIter<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ExactSizeIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, A> FusedIterator for ErasedSoaRefsMutIter<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
