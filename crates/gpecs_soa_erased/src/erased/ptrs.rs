use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaMutPtrs, ErasedSoaRefs, assert::assert_descriptors, error::ErasedSoaIntoValueError,
    },
    error::{check_layout, check_len},
    field::ErasedFieldPtr,
    soa::traits::{FieldDescriptor, Soa},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaPtrs<'context> {
    descriptors: &'context [FieldDescriptor],
    buffer: *const u8,
    capacity: usize,
    offset: usize,
}

impl<'context> ErasedSoaPtrs<'context> {
    #[inline]
    pub unsafe fn new(
        descriptors: &'context [FieldDescriptor],
        buffer: *const u8,
        capacity: usize,
        offset: usize,
    ) -> Self {
        Self {
            descriptors,
            buffer,
            capacity,
            offset,
        }
    }

    #[inline]
    pub fn dangling(descriptors: &'context [FieldDescriptor]) -> Self {
        let addr = descriptors
            .iter()
            .map(|desc| desc.layout().align())
            .max()
            .unwrap_or(1);
        let buffer = ptr::without_provenance(addr);

        let packed_size = descriptors
            .iter()
            .map(|desc| desc.layout().size())
            .sum::<usize>();
        let capacity = match packed_size {
            0 => usize::MAX,
            _ => 0,
        };

        Self {
            descriptors,
            buffer,
            capacity,
            offset: 0,
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
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::Ptrs<'_>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
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
            return Err(ErasedSoaIntoValueError::new(self, error));
        }

        unsafe {
            let ptrs = T::ptrs_from_buffer(context, buffer.cast_mut(), capacity);
            let ptrs = T::ptrs_add_mut(context, ptrs, offset);
            let ptrs = T::ptrs_cast_const(context, ptrs);
            Ok(ptrs)
        }
    }

    #[inline]
    pub fn into_parts(self) -> (&'context [FieldDescriptor], *const u8, usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedSoaMutPtrs<'context> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        let buffer = buffer.cast_mut();
        unsafe { ErasedSoaMutPtrs::new(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { offset, .. } = self;
        Self {
            offset: unsafe { offset.unchecked_add(count) },
            ..self
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: ErasedSoaPtrs<'_>) -> isize {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        assert_eq!(buffer, origin.buffer);
        assert_eq!(capacity, origin.capacity);
        assert_descriptors(descriptors, origin.field_descriptors());

        unsafe { (offset - origin.offset).try_into().unwrap_unchecked() }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaRefs<'context, 'a> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, capacity, offset) }
    }
}

impl<'context> IntoIterator for ErasedSoaPtrs<'context> {
    type Item = ErasedFieldPtr;
    type IntoIter = ErasedSoaPtrsIter<'context>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        ErasedSoaPtrsIter {
            descriptors: descriptors.iter(),
            buffer,
            capacity,
            offset,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaPtrsIter<'context> {
    descriptors: slice::Iter<'context, FieldDescriptor>,
    buffer: *const u8,
    capacity: usize,
    offset: usize,
}

impl ErasedSoaPtrsIter<'_> {
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
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }
}

impl Debug for ErasedSoaPtrsIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_list().entries(entries).finish()
    }
}

impl Iterator for ErasedSoaPtrsIter<'_> {
    type Item = ErasedFieldPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut buffer,
            capacity,
            offset,
        } = *self;

        let &desc = descriptors.next()?;
        let ptr_buffer = ptr::slice_from_raw_parts(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, ptr_buffer) };

        let item = unsafe { ptr.add(offset) };
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

impl ExactSizeIterator for ErasedSoaPtrsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }
}

impl FusedIterator for ErasedSoaPtrsIter<'_> {}
