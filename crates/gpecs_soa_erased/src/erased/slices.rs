use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaSlicePtrs,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError, check_sufficient_len},
    },
    error::{check_layout, check_len},
    field::{ErasedFieldPtr, ErasedFieldSlice, field_slice_from_raw_parts},
    soa::{
        field::{FieldDescriptor, buffer_layout},
        traits::{RawSoaContext, Soa},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSlices<'a, D>
where
    D: ?Sized,
{
    buffer: *const u8,
    capacity: usize,
    offset: usize,
    len: usize,
    phantom: PhantomData<&'a [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaSlices<'_, D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: *const u8,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        Self {
            descriptors,
            buffer,
            capacity,
            offset,
            len,
            phantom: PhantomData,
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
            ..
        } = self;
        (descriptors, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaSlicePtrs<D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            len,
            ..
        } = self;
        unsafe { ErasedSoaSlicePtrs::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }
}

impl<'a, D> ErasedSoaSlices<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    // TODO: check offset & len to be smaller than capacity
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: &'a [u8],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, ErasedSoaPtrsError> {
        let layout = buffer_layout(descriptors.as_ref(), capacity)?;
        check_sufficient_len(buffer.len(), layout.size())?;

        let buffer = buffer.as_ptr();
        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset, len) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::Slices<'_, 'a>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            len,
            ..
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
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        Ok(slices)
    }
}

impl<D> ErasedSoaSlices<'_, D>
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

impl<D> ErasedSoaSlices<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicesIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            len,
            ..
        } = *self;

        ErasedSoaSlicesIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            offset,
            len,
            phantom: PhantomData,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaSlices<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSlice<'a>;
    type IntoIter = ErasedSoaSlicesIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedSoaSlices<'a, D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldSlice<'a>;
    type IntoIter = ErasedSoaSlicesIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            len,
            phantom,
        } = self;

        ErasedSoaSlicesIter {
            descriptors: descriptors.into_iter(),
            buffer,
            capacity,
            offset,
            len,
            phantom,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaSlicesIter<'a, D>
where
    D: ?Sized,
{
    buffer: *const u8,
    capacity: usize,
    offset: usize,
    len: usize,
    phantom: PhantomData<&'a [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaSlicesIter<'_, D>
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

impl<D> ErasedSoaSlicesIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D> Debug for ErasedSoaSlicesIter<'_, D>
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
            ..
        } = *self;

        let entries = ErasedSoaSlicesIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            offset,
            len,
            phantom: PhantomData,
        };
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D> Iterator for ErasedSoaSlicesIter<'a, D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldSlice<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut buffer,
            capacity,
            offset,
            len,
            ..
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let ptr_buffer = ptr::slice_from_raw_parts(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, ptr_buffer) };

        let item = unsafe { field_slice_from_raw_parts(ptr.add(offset), len).deref() };
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

impl<D> ExactSizeIterator for ErasedSoaSlicesIter<'_, D>
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

impl<D> FusedIterator for ErasedSoaSlicesIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
