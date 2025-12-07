use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    slice,
};

use crate::{
    erased::{
        ErasedSoaSliceMutPtrs, ErasedSoaSliceMutPtrsIter, ErasedSoaSlicePtrs,
        error::{ErasedSoaIntoValueError, ErasedSoaSlicePtrsError},
    },
    field::ErasedFieldSliceMut,
    soa::{field::FieldDescriptor, traits::Soa},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSlicesMut<'a, D>
where
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [u8]>,
    ptrs: ErasedSoaSliceMutPtrs<D>,
}

impl<D> ErasedSoaSlicesMut<'_, D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *mut u8,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs = unsafe {
            ErasedSoaSliceMutPtrs::new_unchecked(descriptors, ptr, capacity, offset, len)
        };
        unsafe { Self::from_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaSliceMutPtrs<D>) -> Self {
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *mut u8, usize, usize, usize) {
        let Self { ptrs, .. } = self;
        ptrs.into_parts()
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaSlicePtrs<D> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaSliceMutPtrs<D> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<'a, D> ErasedSoaSlicesMut<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new<R>(
        descriptors: D,
        buffer: &'a mut [u8],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, ErasedSoaSlicePtrsError> {
        let ptrs = ErasedSoaSliceMutPtrs::new(descriptors, buffer, capacity, offset, len)?;
        let me = unsafe { Self::from_ptrs(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::SlicesMut<'_, 'a>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = result.map_err(|err| err.map_value(into_self))?;

        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        Ok(slices)
    }
}

impl<D> ErasedSoaSlicesMut<'_, D>
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

impl<D> ErasedSoaSlicesMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicesMutIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self { ptrs, .. } = self;
        ErasedSoaSlicesMutIter {
            ptrs: ptrs.iter(),
            phantom: PhantomData,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaSlicesMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSliceMut<'a>;
    type IntoIter = ErasedSoaSlicesMutIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedSoaSlicesMut<'a, D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldSliceMut<'a>;
    type IntoIter = ErasedSoaSlicesMutIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, phantom } = self;
        ErasedSoaSlicesMutIter {
            ptrs: ptrs.into_iter(),
            phantom,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaSlicesMutIter<'a, D>
where
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [u8]>,
    ptrs: ErasedSoaSliceMutPtrsIter<D>,
}

impl<D> ErasedSoaSlicesMutIter<'_, D>
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

impl<D> ErasedSoaSlicesMutIter<'_, D>
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
    ) -> ErasedSoaSlicesMutIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self { ptrs, .. } = self;
        ErasedSoaSlicesMutIter {
            ptrs: ptrs.field_descriptors_iter(),
            phantom: PhantomData,
        }
    }
}

impl<D> Debug for ErasedSoaSlicesMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.field_descriptors_iter();
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D> Iterator for ErasedSoaSlicesMutIter<'a, D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldSliceMut<'a>;

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

impl<D> ExactSizeIterator for ErasedSoaSlicesMutIter<'_, D>
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

impl<D> FusedIterator for ErasedSoaSlicesMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
