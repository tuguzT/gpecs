use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::{Range, RangeBounds},
    ptr, slice,
};

use crate::{
    erased::{ErasedSoaPtrs, ErasedSoaSliceMutPtrs, ErasedSoaSlices, error::IntoValueError},
    error::{check_layout, check_len},
    field::{ErasedFieldPtr, ErasedFieldSlicePtr, field_slice_from_raw_parts},
    soa::{
        slice::range,
        traits::{FieldDescriptor, Soa},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSlicePtrs<'context> {
    descriptors: &'context [FieldDescriptor],
    buffer: *const u8,
    capacity: usize,
    start: usize,
    end: usize,
}

impl<'context> ErasedSoaSlicePtrs<'context> {
    #[inline]
    pub unsafe fn new<R>(
        descriptors: &'context [FieldDescriptor],
        buffer: *const u8,
        capacity: usize,
        range: R,
    ) -> Self
    where
        R: RangeBounds<usize>,
    {
        let Range { start, end } = self::range(range, ..capacity);
        Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        }
    }

    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = *self;
        descriptors
    }

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
    pub fn range(&self) -> Range<usize> {
        let Self { start, end, .. } = *self;
        start..end
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.range().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::SlicePtrs<'_>, IntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        } = self;

        let result = T::field_descriptors(context)
            .into_iter()
            .zip(self)
            .try_fold(0, |len, (desc, slice)| {
                check_layout(slice.descriptor().layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_len(len, descriptors.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(IntoValueError::new(self, error));
        }

        unsafe {
            let ptrs = T::ptrs_from_buffer(context, buffer.cast_mut(), capacity);
            let ptrs = T::ptrs_add_mut(context, ptrs, start);
            let ptrs = T::ptrs_cast_const(context, ptrs);
            let slices = T::slices_from_raw_parts(context, ptrs, (start..end).len());
            Ok(slices)
        }
    }

    #[inline]
    pub fn into_parts(self) -> (&'context [FieldDescriptor], *const u8, usize, Range<usize>) {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        } = self;
        (descriptors, buffer, capacity, start..end)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedSoaSliceMutPtrs<'context> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        } = self;

        let buffer = buffer.cast_mut();
        unsafe { ErasedSoaSliceMutPtrs::new(descriptors, buffer, capacity, start..end) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaSlices<'context, 'a> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        } = self;
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, buffer, capacity, start..end) }
    }

    #[inline]
    pub fn as_ptrs(&self) -> ErasedSoaPtrs<'context> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            ..
        } = *self;
        unsafe { ErasedSoaPtrs::new(descriptors, buffer, capacity, start) }
    }
}

impl<'context> IntoIterator for ErasedSoaSlicePtrs<'context> {
    type Item = ErasedFieldSlicePtr;
    type IntoIter = ErasedSoaSlicePtrsIter<'context>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        } = self;

        ErasedSoaSlicePtrsIter {
            descriptors: descriptors.iter(),
            buffer,
            capacity,
            start,
            end,
        }
    }
}

#[inline]
pub fn soa_slice_from_raw_parts(data: ErasedSoaPtrs<'_>, len: usize) -> ErasedSoaSlicePtrs<'_> {
    let (descriptors, buffer, capacity, start) = data.into_parts();
    let end = start.checked_add(len).unwrap();
    unsafe { ErasedSoaSlicePtrs::new(descriptors, buffer, capacity, start..end) }
}

#[derive(Clone)]
pub struct ErasedSoaSlicePtrsIter<'context> {
    descriptors: slice::Iter<'context, FieldDescriptor>,
    buffer: *const u8,
    capacity: usize,
    start: usize,
    end: usize,
}

impl ErasedSoaSlicePtrsIter<'_> {
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_slice()
    }

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
    pub fn range(&self) -> Range<usize> {
        let Self { start, end, .. } = *self;
        start..end
    }
}

impl Debug for ErasedSoaSlicePtrsIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_list().entries(entries).finish()
    }
}

impl Iterator for ErasedSoaSlicePtrsIter<'_> {
    type Item = ErasedFieldSlicePtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut buffer,
            capacity,
            start,
            end,
        } = *self;

        let &desc = descriptors.next()?;
        let ptr_buffer = ptr::slice_from_raw_parts(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, ptr_buffer) };

        let item = field_slice_from_raw_parts(unsafe { ptr.add(start) }, (start..end).len());
        *buffer = unsafe { ptr.add(capacity) }.as_ptr();

        if let [desc, ..] = descriptors.as_slice() {
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

impl ExactSizeIterator for ErasedSoaSlicePtrsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }
}

impl FusedIterator for ErasedSoaSlicePtrsIter<'_> {}
