use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaPtrs, ErasedSoaSliceMutPtrs, ErasedSoaSlices, error::ErasedSoaIntoValueError,
    },
    error::{check_layout, check_len},
    field::{ErasedFieldPtr, ErasedFieldSlicePtr, field_slice_from_raw_parts},
    soa::{
        field::FieldDescriptor,
        traits::{RawSoaContext, SlicePtrs, Soa},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSlicePtrs<D>
where
    D: ?Sized,
{
    buffer: *const u8,
    capacity: usize,
    offset: usize,
    len: usize,
    descriptors: D,
}

impl<D> ErasedSoaSlicePtrs<D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: *const u8,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        Self {
            buffer,
            capacity,
            offset,
            len,
            descriptors,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *const u8, usize, usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            len,
        } = self;
        (descriptors, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedSoaSliceMutPtrs<D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            len,
        } = self;

        let buffer = buffer.cast_mut();
        unsafe { ErasedSoaSliceMutPtrs::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaSlices<'a, D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            len,
        } = self;
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }
}

impl<D> ErasedSoaSlicePtrs<D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<SlicePtrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            len,
        } = self;
        let descriptors = descriptors.as_ref();

        let result = context
            .field_descriptors()
            .into_iter()
            .zip(&self)
            .try_fold(0, |len, (desc, slice)| {
                check_layout(slice.descriptor().layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_len(len, descriptors.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(ErasedSoaIntoValueError::new(self, error));
        }

        let ptrs = unsafe { context.ptrs_from_buffer(buffer, capacity) };
        let ptrs = unsafe { context.ptrs_add(ptrs, offset) };
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D> ErasedSoaSlicePtrs<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn buffer(&self) -> *const u8 {
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
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicePtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            len,
        } = *self;

        ErasedSoaSlicePtrsIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            offset,
            len,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaSlicePtrs<D>
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

impl<D> IntoIterator for ErasedSoaSlicePtrs<D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldSlicePtr;
    type IntoIter = ErasedSoaSlicePtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            len,
        } = self;

        ErasedSoaSlicePtrsIter {
            descriptors: descriptors.into_iter(),
            buffer,
            capacity,
            offset,
            len,
        }
    }
}

#[inline]
pub fn slice_from_raw_parts<D>(data: ErasedSoaPtrs<D>, len: usize) -> ErasedSoaSlicePtrs<D> {
    let (descriptors, buffer, capacity, offset) = data.into_parts();
    unsafe { ErasedSoaSlicePtrs::new_unchecked(descriptors, buffer, capacity, offset, len) }
}

#[derive(Clone)]
pub struct ErasedSoaSlicePtrsIter<D>
where
    D: ?Sized,
{
    buffer: *const u8,
    capacity: usize,
    offset: usize,
    len: usize,
    descriptors: D,
}

impl<D> ErasedSoaSlicePtrsIter<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn buffer(&self) -> *const u8 {
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

impl<D> ErasedSoaSlicePtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D> Debug for ErasedSoaSlicePtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            len,
        } = *self;

        let entries = ErasedSoaSlicePtrsIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            offset,
            len,
        };
        f.debug_list().entries(entries).finish()
    }
}

impl<D> Iterator for ErasedSoaSlicePtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldSlicePtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut buffer,
            capacity,
            offset,
            len,
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let ptr_buffer = ptr::slice_from_raw_parts(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, ptr_buffer) };

        let item = field_slice_from_raw_parts(unsafe { ptr.add(offset) }, len);
        *buffer = unsafe { ptr.add(capacity) }.as_ptr();

        if let [desc, ..] = descriptors.as_ref() {
            *buffer = unsafe { buffer.add(buffer.align_offset(desc.layout().align())) };
        }
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { descriptors, .. } = self;
        descriptors.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedSoaSlicePtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ExactSizeIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }
}

impl<D> FusedIterator for ErasedSoaSlicePtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
