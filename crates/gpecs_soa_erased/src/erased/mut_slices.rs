use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaSliceMutPtrs, ErasedSoaSliceMutPtrsIter, ErasedSoaSlicePtrs, ErasedSoaSlicesIter,
        error::{ErasedSoaIntoValueError, ErasedSoaSlicePtrsError},
    },
    field::{ErasedFieldSlice, ErasedFieldSliceMut},
    soa::{
        field::FieldDescriptor,
        traits::{AllocSoa, SlicesMut, Soa, SoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaSlicesMut<'a, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [MaybeUninit<A>]>,
    ptrs: ErasedSoaSliceMutPtrs<D, A>,
}

impl<'a, D, A> ErasedSoaSlicesMut<'a, D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: &'a mut [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let buffer = ptr::from_mut(buffer) as _;
        let ptrs = unsafe {
            ErasedSoaSliceMutPtrs::new_unchecked(descriptors, buffer, capacity, offset, len)
        };

        unsafe { Self::from_mut_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(ptrs: ErasedSoaSliceMutPtrs<D, A>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a mut [MaybeUninit<A>], usize, usize, usize) {
        let Self { ptrs, .. } = self;
        let (descriptors, buffer, capacity, offset, len) = ptrs.into_parts();

        let buffer = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        (descriptors, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaSlicePtrs<D, A> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaSliceMutPtrs<D, A> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<'a, D, A> ErasedSoaSlicesMut<'a, D, A>
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
        len: usize,
    ) -> Result<Self, ErasedSoaSlicePtrsError> {
        let buffer = ptr::from_mut(buffer) as _;
        let ptrs = ErasedSoaSliceMutPtrs::new(descriptors, buffer, capacity, offset, len)?;

        let me = unsafe { Self::from_mut_ptrs(ptrs) };
        Ok(me)
    }
}

impl<'a, D> ErasedSoaSlicesMut<'a, D, u8>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<SlicesMut<'_, 'a, T>, ErasedSoaIntoValueError<Self>>
    where
        T: AllocSoa + Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_mut_ptrs(ptrs) };
        let slices = result.map_err(|err| err.map_value(into_self))?;

        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        Ok(slices)
    }
}

impl<D, A> ErasedSoaSlicesMut<'_, D, A>
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

    #[inline]
    pub fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<D, A> ErasedSoaSlicesMut<'_, D, A>
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
    pub fn iter(&self) -> ErasedSoaSlicesIter<'_, slice::Iter<'_, FieldDescriptor>, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicesIter::from_ptrs(ptrs) }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedSoaSlicesMutIter<'_, slice::Iter<'_, FieldDescriptor>, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter_mut();
        unsafe { ErasedSoaSlicesMutIter::from_mut_ptrs(ptrs) }
    }
}

impl<D, A> Debug for ErasedSoaSlicesMut<'_, D, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_struct("ErasedSoaSlicesMut")
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<'a, D, A> IntoIterator for &'a ErasedSoaSlicesMut<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSlice<'a, A>;
    type IntoIter = ErasedSoaSlicesIter<'a, slice::Iter<'a, FieldDescriptor>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, A> IntoIterator for &'a mut ErasedSoaSlicesMut<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSliceMut<'a, A>;
    type IntoIter = ErasedSoaSlicesMutIter<'a, slice::Iter<'a, FieldDescriptor>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D, A> IntoIterator for ErasedSoaSlicesMut<'a, D, A>
where
    A: AddressableUnit,
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldSliceMut<'a, A>;
    type IntoIter = ErasedSoaSlicesMutIter<'a, D::IntoIter, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaSlicesMutIter::from_mut_ptrs(ptrs) }
    }
}

pub struct ErasedSoaSlicesMutIter<'a, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [MaybeUninit<A>]>,
    ptrs: ErasedSoaSliceMutPtrsIter<D, A>,
}

impl<D, A> ErasedSoaSlicesMutIter<'_, D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn from_mut_ptrs(ptrs: ErasedSoaSliceMutPtrsIter<D, A>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, A> ErasedSoaSlicesMutIter<'_, D, A>
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

impl<D, A> ErasedSoaSlicesMutIter<'_, D, A>
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
    ) -> ErasedSoaSlicesMutIter<'_, slice::Iter<'_, FieldDescriptor>, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.debug_entries();
        unsafe { ErasedSoaSlicesMutIter::from_mut_ptrs(ptrs) }
    }
}

impl<D, A> Debug for ErasedSoaSlicesMutIter<'_, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.debug_entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D, A> Iterator for ErasedSoaSlicesMutIter<'a, D, A>
where
    A: AddressableUnit,
    D: Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldSliceMut<'a, A>;

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

impl<D, A> ExactSizeIterator for ErasedSoaSlicesMutIter<'_, D, A>
where
    A: AddressableUnit,
    D: ExactSizeIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, A> FusedIterator for ErasedSoaSlicesMutIter<'_, D, A>
where
    A: AddressableUnit,
    D: FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
