use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    assert::{check_same_layout, check_same_len},
    erased::{ErasedSoaMutPtrs, ErasedSoaPtrs, error::IntoValueError},
    field::{ErasedFieldMutPtr, ErasedFieldRefMut},
    soa::traits::{FieldDescriptor, Soa, buffer_layout},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaRefsMut<'context, 'a> {
    descriptors: &'context [FieldDescriptor],
    buffer: *mut u8,
    capacity: usize,
    offset: usize,
    phantom: PhantomData<&'a mut [u8]>,
}

impl<'context, 'a> ErasedSoaRefsMut<'context, 'a> {
    #[inline]
    #[track_caller]
    pub fn new(
        descriptors: &'context [FieldDescriptor],
        buffer: &'a mut [u8],
        capacity: usize,
        offset: usize,
    ) -> Self {
        let layout = buffer_layout(descriptors, capacity)
            .expect("buffer layout size should not exceed `isize::MAX`");
        assert!(
            buffer.len() >= layout.size(),
            "buffer length ({buffer_len}) should be equal to or larger than expected layout size ({layout_size})",
            buffer_len = buffer.len(),
            layout_size = layout.size(),
        );

        let buffer = buffer.as_mut_ptr();
        unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: &'context [FieldDescriptor],
        buffer: *mut u8,
        capacity: usize,
        offset: usize,
    ) -> Self {
        Self {
            descriptors,
            buffer,
            capacity,
            offset,
            phantom: PhantomData,
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
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::RefsMut<'_, 'a>, IntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
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
            let ptrs = T::ptrs_add_mut(context, ptrs, offset);
            let refs = T::ptrs_to_refs_mut(context, ptrs);
            Ok(refs)
        }
    }

    #[inline]
    pub fn into_parts(self) -> (&'context [FieldDescriptor], *mut u8, usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn as_ptr(&self) -> ErasedSoaPtrs<'context> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;
        unsafe { ErasedSoaPtrs::new(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> ErasedSoaMutPtrs<'context> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;
        unsafe { ErasedSoaMutPtrs::new(descriptors, buffer, capacity, offset) }
    }
}

impl<'context, 'a> IntoIterator for ErasedSoaRefsMut<'context, 'a> {
    type Item = ErasedFieldRefMut<'a>;
    type IntoIter = ErasedSoaRefsMutIter<'context, 'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            phantom,
        } = self;

        ErasedSoaRefsMutIter {
            descriptors: descriptors.iter(),
            buffer,
            capacity,
            offset,
            phantom,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaRefsMutIter<'context, 'a> {
    descriptors: slice::Iter<'context, FieldDescriptor>,
    buffer: *mut u8,
    capacity: usize,
    offset: usize,
    phantom: PhantomData<&'a mut [u8]>,
}

impl ErasedSoaRefsMutIter<'_, '_> {
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
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }
}

impl Debug for ErasedSoaRefsMutIter<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_list().entries(entries).finish()
    }
}

impl<'a> Iterator for ErasedSoaRefsMutIter<'_, 'a> {
    type Item = ErasedFieldRefMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut buffer,
            capacity,
            offset,
            ..
        } = *self;

        let &desc = descriptors.next()?;
        let ptr_buffer = ptr::slice_from_raw_parts_mut(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, ptr_buffer) };

        let item = unsafe { ptr.add(offset).deref_mut() };
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

impl ExactSizeIterator for ErasedSoaRefsMutIter<'_, '_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }
}

impl FusedIterator for ErasedSoaRefsMutIter<'_, '_> {}
