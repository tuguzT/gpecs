use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    slice,
};

use crate::{
    erased::{
        ErasedSoaPtrs, ErasedSoaPtrsIter, ErasedSoaSliceMutPtrs, ErasedSoaSlices,
        error::{ErasedSoaIntoValueError, ErasedSoaSlicePtrsError, check_offset, check_offset_len},
    },
    error::check_sufficient_len,
    field::{ErasedFieldSlicePtr, field_slice_from_raw_parts},
    soa::{
        field::{FieldDescriptor, buffer_layout},
        traits::{RawSoa, RawSoaContext, SlicePtrs},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSlicePtrs<D>
where
    D: ?Sized,
{
    len: usize,
    ptrs: ErasedSoaPtrs<D>,
}

impl<D> ErasedSoaSlicePtrs<D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *const u8,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaPtrs::new_unchecked(descriptors, ptr, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaPtrs<D>, len: usize) -> Self {
        Self { len, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *const u8, usize, usize, usize) {
        let Self { ptrs, len } = self;
        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        (descriptors, ptr, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedSoaSliceMutPtrs<D> {
        let Self { ptrs, len } = self;
        unsafe { ErasedSoaSliceMutPtrs::from_mut_ptrs(ptrs.cast_mut(), len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaSlices<'a, D> {
        let Self { ptrs, len } = self;
        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, ptr, capacity, offset, len) }
    }
}

impl<D> ErasedSoaSlicePtrs<D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: *const [u8],
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
    ) -> Result<SlicePtrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: RawSoa + ?Sized,
    {
        let Self { ptrs, len } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D> ErasedSoaSlicePtrs<D>
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

impl<D> ErasedSoaSlicePtrs<D>
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
}

impl<'a, D> IntoIterator for &'a ErasedSoaSlicePtrs<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSlicePtr<u8>;
    type IntoIter = ErasedSoaSlicePtrsIter<slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> IntoIterator for ErasedSoaSlicePtrs<D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldSlicePtr<u8>;
    type IntoIter = ErasedSoaSlicePtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, len } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

#[inline]
pub unsafe fn slice_from_raw_parts<D>(data: ErasedSoaPtrs<D>, len: usize) -> ErasedSoaSlicePtrs<D> {
    unsafe { ErasedSoaSlicePtrs::from_ptrs(data, len) }
}

#[derive(Clone)]
pub struct ErasedSoaSlicePtrsIter<D>
where
    D: ?Sized,
{
    len: usize,
    ptrs: ErasedSoaPtrsIter<D>,
}

impl<D> ErasedSoaSlicePtrsIter<D> {
    #[inline]
    pub(super) unsafe fn new_unchecked(ptrs: ErasedSoaPtrsIter<D>, len: usize) -> Self {
        Self { len, ptrs }
    }
}

impl<D> ErasedSoaSlicePtrsIter<D>
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

impl<D> ErasedSoaSlicePtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }

    #[inline]
    pub(super) fn debug_entries(&self) -> ErasedSoaSlicePtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.debug_entries();
        unsafe { ErasedSoaSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D> Debug for ErasedSoaSlicePtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.debug_entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D> Iterator for ErasedSoaSlicePtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldSlicePtr<u8>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ref mut ptrs, len } = *self;

        let data = ptrs.next()?;
        let item = unsafe { field_slice_from_raw_parts(data, len) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedSoaSlicePtrsIter<D>
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

impl<D> FusedIterator for ErasedSoaSlicePtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
