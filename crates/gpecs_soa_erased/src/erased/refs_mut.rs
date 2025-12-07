use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    slice,
};

use crate::{
    erased::{
        ErasedSoaMutPtrs, ErasedSoaMutPtrsIter, ErasedSoaPtrs,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError},
    },
    field::ErasedFieldRefMut,
    soa::{field::FieldDescriptor, traits::Soa},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaRefsMut<'a, D>
where
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [u8]>,
    ptrs: ErasedSoaMutPtrs<D>,
}

impl<D> ErasedSoaRefsMut<'_, D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *mut u8,
        capacity: usize,
        offset: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, ptr, capacity, offset) };
        unsafe { Self::from_mut_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(ptrs: ErasedSoaMutPtrs<D>) -> Self {
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *mut u8, usize, usize) {
        let Self { ptrs, .. } = self;
        ptrs.into_parts()
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaMutPtrs<D> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<'a, D> ErasedSoaRefsMut<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: &'a mut [u8],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, ErasedSoaPtrsError> {
        let ptrs = ErasedSoaMutPtrs::new(descriptors, buffer, capacity, offset)?;
        let me = unsafe { Self::from_mut_ptrs(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::RefsMut<'_, 'a>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_mut_ptrs(ptrs) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let refs = unsafe { T::ptrs_to_refs_mut(context, ptrs) };
        Ok(refs)
    }
}

impl<D> ErasedSoaRefsMut<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptrs, .. } = self;
        ptrs.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { ptrs, .. } = self;
        ptrs.as_mut_ptr()
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

impl<D> ErasedSoaRefsMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaRefsMutIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self { ptrs, .. } = self;
        ErasedSoaRefsMutIter {
            ptrs: ptrs.iter(),
            phantom: PhantomData,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaRefsMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldRefMut<'a>;
    type IntoIter = ErasedSoaRefsMutIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedSoaRefsMut<'a, D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldRefMut<'a>;
    type IntoIter = ErasedSoaRefsMutIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, phantom } = self;
        ErasedSoaRefsMutIter {
            ptrs: ptrs.into_iter(),
            phantom,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaRefsMutIter<'a, D>
where
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [u8]>,
    ptrs: ErasedSoaMutPtrsIter<D>,
}

impl<D> ErasedSoaRefsMutIter<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptrs, .. } = self;
        ptrs.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { ptrs, .. } = self;
        ptrs.as_mut_ptr()
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

impl<D> ErasedSoaRefsMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }

    #[inline]
    pub fn field_descriptors_iter(
        &self,
    ) -> ErasedSoaRefsMutIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self { ptrs, .. } = self;
        ErasedSoaRefsMutIter {
            ptrs: ptrs.field_descriptors_iter(),
            phantom: PhantomData,
        }
    }
}

impl<D> Debug for ErasedSoaRefsMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.field_descriptors_iter();
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D> Iterator for ErasedSoaRefsMutIter<'a, D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldRefMut<'a>;

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

impl<D> ExactSizeIterator for ErasedSoaRefsMutIter<'_, D>
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

impl<D> FusedIterator for ErasedSoaRefsMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
