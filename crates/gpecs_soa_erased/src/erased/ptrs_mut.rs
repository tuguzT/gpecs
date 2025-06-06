use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr, slice,
};

use crate::{
    assert::{check_same_layout, check_same_len},
    erased::{
        assert::assert_descriptors, error::IntoValueError, ErasedSoaPtrs, ErasedSoaPtrsIter,
        ErasedSoaRefs, ErasedSoaRefsMut,
    },
    field::ErasedFieldMutPtr,
    soa::traits::{FieldDescriptor, Soa},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaMutPtrs<'context> {
    descriptors: &'context [FieldDescriptor],
    buffer: *mut u8,
    capacity: usize,
    offset: usize,
}

impl<'context> ErasedSoaMutPtrs<'context> {
    #[inline]
    pub unsafe fn new(
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
        }
    }

    #[inline]
    pub fn dangling(descriptors: &'context [FieldDescriptor]) -> Self {
        let addr = descriptors
            .iter()
            .map(|desc| desc.layout().align())
            .max()
            .unwrap_or(1);
        let buffer = ptr::without_provenance_mut(addr);

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
    ) -> Result<T::MutPtrs<'_>, IntoValueError<Self>>
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
            Ok(ptrs)
        }
    }

    #[inline]
    pub fn into_parts(self) -> (&'context [FieldDescriptor], *mut u8, usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSoaPtrs<'context> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        let buffer = buffer.cast_const();
        unsafe { ErasedSoaPtrs::new(descriptors, buffer, capacity, offset) }
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

        assert_eq!(buffer.cast_const(), origin.buffer());
        assert_eq!(capacity, origin.capacity());
        assert_descriptors(descriptors, origin.field_descriptors());

        unsafe { (offset - origin.offset()).try_into().unwrap_unchecked() }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap(self, with: ErasedSoaMutPtrs<'_>, temp: &mut [u8]) {
        let Self { descriptors, .. } = self;
        assert_descriptors(descriptors, with.field_descriptors());

        itertools::zip_eq(self, with).for_each(|(this, with)| unsafe { this.swap(with, temp) })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from(self, from: ErasedSoaPtrs<'_>, count: usize, temp: &mut [u8]) {
        let Self { descriptors, .. } = self;
        assert_descriptors(descriptors, from.field_descriptors());

        itertools::zip_eq(self, from)
            .for_each(|(this, from)| unsafe { this.copy_from(from, count, temp) })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_rev(self, from: ErasedSoaPtrs<'_>, count: usize, temp: &mut [u8]) {
        let Self { descriptors, .. } = self;
        assert_descriptors(descriptors, from.field_descriptors());

        #[inline]
        #[track_caller]
        fn rec(
            iter: &mut itertools::ZipEq<ErasedSoaMutPtrsIter<'_>, ErasedSoaPtrsIter<'_>>,
            count: usize,
            temp: &mut [u8],
        ) {
            let Some((this, from)) = iter.next() else {
                return;
            };
            rec(iter, count, temp);
            unsafe { this.copy_from(from, count, temp) }
        }

        let mut iter = itertools::zip_eq(self, from);
        rec(&mut iter, count, temp)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(self, from: ErasedSoaPtrs<'_>, count: usize) {
        let Self { descriptors, .. } = self;
        assert_descriptors(descriptors, from.field_descriptors());

        itertools::zip_eq(self, from)
            .for_each(|(this, from)| unsafe { this.copy_from_nonoverlapping(from, count) })
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

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaRefsMut<'context, 'a> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        unsafe { ErasedSoaRefsMut::new_unchecked(descriptors, buffer, capacity, offset) }
    }
}

impl<'context> IntoIterator for ErasedSoaMutPtrs<'context> {
    type Item = ErasedFieldMutPtr;
    type IntoIter = ErasedSoaMutPtrsIter<'context>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        ErasedSoaMutPtrsIter {
            descriptors: descriptors.iter(),
            buffer,
            capacity,
            offset,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaMutPtrsIter<'context> {
    descriptors: slice::Iter<'context, FieldDescriptor>,
    buffer: *mut u8,
    capacity: usize,
    offset: usize,
}

impl ErasedSoaMutPtrsIter<'_> {
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

impl Debug for ErasedSoaMutPtrsIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_list().entries(entries).finish()
    }
}

impl Iterator for ErasedSoaMutPtrsIter<'_> {
    type Item = ErasedFieldMutPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut buffer,
            capacity,
            offset,
        } = *self;

        let &desc = descriptors.next()?;
        let ptr_buffer = ptr::slice_from_raw_parts_mut(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, ptr_buffer) };

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

impl ExactSizeIterator for ErasedSoaMutPtrsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }
}

impl FusedIterator for ErasedSoaMutPtrsIter<'_> {}
