use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::{Range, RangeBounds},
    ptr, slice,
};

use crate::{
    assert::{check_same_layout, check_same_len},
    erased::{
        error::IntoValueError, ErasedSoaMutPtrs, ErasedSoaPtrs, ErasedSoaSlicePtrs,
        ErasedSoaSlices, ErasedSoaSlicesMut,
    },
    field::{field_slice_from_raw_parts_mut, ErasedFieldMutPtr, ErasedFieldSliceMutPtr},
    soa::{
        slice::range,
        traits::{FieldDescriptor, Soa},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSliceMutPtrs<'context> {
    descriptors: &'context [FieldDescriptor],
    buffer: *mut u8,
    capacity: usize,
    start: usize,
    end: usize,
}

impl<'context> ErasedSoaSliceMutPtrs<'context> {
    #[inline]
    pub unsafe fn new<R>(
        descriptors: &'context [FieldDescriptor],
        buffer: *mut u8,
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
    pub fn buffer(&self) -> *mut u8 {
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
    ) -> Result<T::SliceMutPtrs<'_>, IntoValueError<Self>>
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
                check_same_layout(slice.descriptor().layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_same_len(len, descriptors.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(IntoValueError::new(self, error));
        }

        unsafe {
            let ptrs = T::ptrs_from_buffer(context, buffer, capacity);
            let ptrs = T::ptrs_add_mut(context, ptrs, start);
            let slices = T::slices_from_raw_parts_mut(context, ptrs, (start..end).len());
            Ok(slices)
        }
    }

    #[inline]
    pub fn into_parts(self) -> (&'context [FieldDescriptor], *mut u8, usize, Range<usize>) {
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
    pub fn cast_const(self) -> ErasedSoaSlicePtrs<'context> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        } = self;

        let buffer = buffer.cast_const();
        unsafe { ErasedSoaSlicePtrs::new(descriptors, buffer, capacity, start..end) }
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
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaSlicesMut<'context, 'a> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        } = self;
        unsafe { ErasedSoaSlicesMut::new_unchecked(descriptors, buffer, capacity, start..end) }
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

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> ErasedSoaMutPtrs<'context> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            ..
        } = *self;
        unsafe { ErasedSoaMutPtrs::new(descriptors, buffer, capacity, start) }
    }
}

impl<'context> IntoIterator for ErasedSoaSliceMutPtrs<'context> {
    type Item = ErasedFieldSliceMutPtr;
    type IntoIter = ErasedSoaSliceMutPtrsIter<'context>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        } = self;

        ErasedSoaSliceMutPtrsIter {
            descriptors: descriptors.iter(),
            buffer,
            capacity,
            start,
            end,
        }
    }
}

#[inline]
pub fn soa_slice_from_raw_parts_mut(
    data: ErasedSoaMutPtrs<'_>,
    len: usize,
) -> ErasedSoaSliceMutPtrs<'_> {
    let (descriptors, buffer, capacity, start) = data.into_parts();
    let end = start.checked_add(len).unwrap();
    unsafe { ErasedSoaSliceMutPtrs::new(descriptors, buffer, capacity, start..end) }
}

#[derive(Clone)]
pub struct ErasedSoaSliceMutPtrsIter<'context> {
    descriptors: slice::Iter<'context, FieldDescriptor>,
    buffer: *mut u8,
    capacity: usize,
    start: usize,
    end: usize,
}

impl ErasedSoaSliceMutPtrsIter<'_> {
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_slice()
    }

    #[inline]
    pub fn buffer(&self) -> *mut u8 {
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

impl Debug for ErasedSoaSliceMutPtrsIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_list().entries(entries).finish()
    }
}

impl Iterator for ErasedSoaSliceMutPtrsIter<'_> {
    type Item = ErasedFieldSliceMutPtr;

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
        let ptr_buffer = ptr::slice_from_raw_parts_mut(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, ptr_buffer) };

        let item = field_slice_from_raw_parts_mut(unsafe { ptr.add(start) }, (start..end).len());
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

impl ExactSizeIterator for ErasedSoaSliceMutPtrsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }
}

impl FusedIterator for ErasedSoaSliceMutPtrsIter<'_> {}
