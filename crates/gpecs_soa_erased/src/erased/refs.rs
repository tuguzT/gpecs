use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    slice,
};

use crate::{
    erased::{
        ErasedSoaPtrs, ErasedSoaPtrsIter,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError},
    },
    field::ErasedFieldRef,
    soa::{
        field::FieldDescriptor,
        traits::{Refs, Soa, SoaContext},
    },
};

#[derive(Clone, Copy)]
pub struct ErasedSoaRefs<'a, D>
where
    D: ?Sized,
{
    phantom: PhantomData<&'a [u8]>,
    ptrs: ErasedSoaPtrs<D>,
}

impl<D> ErasedSoaRefs<'_, D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *const u8,
        capacity: usize,
        offset: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaPtrs::new_unchecked(descriptors, ptr, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaPtrs<D>) -> Self {
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *const u8, usize, usize) {
        let Self { ptrs, .. } = self;
        ptrs.into_parts()
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<'a, D> ErasedSoaRefs<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: &'a [u8],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, ErasedSoaPtrsError> {
        let ptrs = ErasedSoaPtrs::new(descriptors, buffer, capacity, offset)?;
        let me = unsafe { Self::from_ptrs(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<Refs<'_, 'a, T>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let refs = unsafe { context.ptrs_to_refs(ptrs) };
        Ok(refs)
    }
}

impl<D> ErasedSoaRefs<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptrs, .. } = self;
        ptrs.as_ptr()
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

impl<D> ErasedSoaRefs<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaRefsIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<D> Debug for ErasedSoaRefs<'_, D>
where
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_struct("ErasedSoaRefs")
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaRefs<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldRef<'a, u8>;
    type IntoIter = ErasedSoaRefsIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedSoaRefs<'a, D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldRef<'a, u8>;
    type IntoIter = ErasedSoaRefsIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

#[derive(Clone)]
pub struct ErasedSoaRefsIter<'a, D>
where
    D: ?Sized,
{
    phantom: PhantomData<&'a [u8]>,
    ptrs: ErasedSoaPtrsIter<D>,
}

impl<D> ErasedSoaRefsIter<'_, D> {
    #[inline]
    pub(super) unsafe fn from_ptrs(ptrs: ErasedSoaPtrsIter<D>) -> Self {
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }
}

impl<D> ErasedSoaRefsIter<'_, D>
where
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

impl<D> ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }

    #[inline]
    pub(super) fn debug_entries(&self) -> ErasedSoaRefsIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.debug_entries();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<D> Debug for ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.debug_entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D> Iterator for ErasedSoaRefsIter<'a, D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldRef<'a, u8>;

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

impl<D> ExactSizeIterator for ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ExactSizeIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D> FusedIterator for ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
