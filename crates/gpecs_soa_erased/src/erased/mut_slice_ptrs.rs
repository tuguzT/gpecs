use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    slice,
};

use crate::{
    erased::{
        ErasedSoaMutPtrs, ErasedSoaMutPtrsIter, ErasedSoaPtrs, ErasedSoaSlicePtrs,
        ErasedSoaSlicePtrsIter, ErasedSoaSlices, ErasedSoaSlicesMut,
        error::{
            ErasedSoaIntoValueError, ErasedSoaSlicePtrsError, check_offset, check_offset_len,
            check_sufficient_len,
        },
    },
    field::{ErasedFieldSliceMutPtr, ErasedFieldSlicePtr, field_slice_from_raw_parts_mut},
    soa::{
        field::{FieldDescriptor, buffer_layout},
        traits::{RawSoa, RawSoaContext, SliceMutPtrs},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSliceMutPtrs<D>
where
    D: ?Sized,
{
    len: usize,
    ptrs: ErasedSoaMutPtrs<D>,
}

impl<D> ErasedSoaSliceMutPtrs<D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *mut u8,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, ptr, capacity, offset) };
        unsafe { Self::from_mut_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(ptrs: ErasedSoaMutPtrs<D>, len: usize) -> Self {
        Self { len, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *mut u8, usize, usize, usize) {
        let Self { ptrs, len } = self;
        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        (descriptors, ptr, capacity, offset, len)
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

    #[inline]
    pub fn cast_const(self) -> ErasedSoaSlicePtrs<D> {
        let Self { ptrs, len } = self;
        unsafe { ErasedSoaSlicePtrs::from_ptrs(ptrs.cast_const(), len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaSlices<'a, D> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaSlicesMut<'a, D> {
        let Self { ptrs, len } = self;
        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        unsafe { ErasedSoaSlicesMut::new_unchecked(descriptors, ptr, capacity, offset, len) }
    }
}

impl<D> ErasedSoaSliceMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: *mut [u8],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, ErasedSoaSlicePtrsError> {
        let layout = buffer_layout(descriptors.as_ref(), capacity)?;
        check_sufficient_len(buffer.len(), layout.size())?;
        check_offset(offset, capacity)?;
        check_offset_len(offset, len, capacity)?;

        let ptr = buffer.cast();
        let me = unsafe { Self::new_unchecked(descriptors, ptr, capacity, offset, len) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<SliceMutPtrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: RawSoa + ?Sized,
    {
        let Self { ptrs, len } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_mut_ptrs(ptrs, len) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let slices = context.mut_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D> ErasedSoaSliceMutPtrs<D>
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
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<D> ErasedSoaSliceMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicePtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicePtrsIter::new_unchecked(ptrs, len) }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedSoaSliceMutPtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self { ref mut ptrs, len } = *self;

        let ptrs = ptrs.iter_mut();
        unsafe { ErasedSoaSliceMutPtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaSliceMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSlicePtr;
    type IntoIter = ErasedSoaSlicePtrsIter<slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for &'a mut ErasedSoaSliceMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSliceMutPtr;
    type IntoIter = ErasedSoaSliceMutPtrsIter<slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D> IntoIterator for ErasedSoaSliceMutPtrs<D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldSliceMutPtr;
    type IntoIter = ErasedSoaSliceMutPtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, len } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaSliceMutPtrsIter::new_unchecked(ptrs, len) }
    }
}

#[inline]
pub unsafe fn slice_from_raw_parts_mut<D>(
    data: ErasedSoaMutPtrs<D>,
    len: usize,
) -> ErasedSoaSliceMutPtrs<D> {
    unsafe { ErasedSoaSliceMutPtrs::from_mut_ptrs(data, len) }
}

#[derive(Clone)]
pub struct ErasedSoaSliceMutPtrsIter<D>
where
    D: ?Sized,
{
    len: usize,
    ptrs: ErasedSoaMutPtrsIter<D>,
}

impl<D> ErasedSoaSliceMutPtrsIter<D> {
    #[inline]
    pub(super) unsafe fn new_unchecked(ptrs: ErasedSoaMutPtrsIter<D>, len: usize) -> Self {
        Self { len, ptrs }
    }
}

impl<D> ErasedSoaSliceMutPtrsIter<D>
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

impl<D> ErasedSoaSliceMutPtrsIter<D>
where
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
    ) -> ErasedSoaSliceMutPtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.debug_entries();
        unsafe { ErasedSoaSliceMutPtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D> Debug for ErasedSoaSliceMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.debug_entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D> Iterator for ErasedSoaSliceMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldSliceMutPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ref mut ptrs, len } = *self;

        let data = ptrs.next()?;
        let item = unsafe { field_slice_from_raw_parts_mut(data, len) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedSoaSliceMutPtrsIter<D>
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

impl<D> FusedIterator for ErasedSoaSliceMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
