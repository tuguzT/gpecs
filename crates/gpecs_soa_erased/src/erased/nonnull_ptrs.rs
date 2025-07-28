use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
    ptr::{self, NonNull},
    slice,
};

use crate::{
    erased::{assert::assert_descriptors, error::ErasedSoaIntoValueError},
    error::{check_layout, check_len},
    field::ErasedFieldNonNullPtr,
    soa::traits::{FieldDescriptor, Soa},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaNonNullPtrs<'context> {
    descriptors: &'context [FieldDescriptor],
    buffer: NonNull<u8>,
    capacity: usize,
    offset: usize,
}

impl<'context> ErasedSoaNonNullPtrs<'context> {
    #[inline]
    pub unsafe fn new(
        descriptors: &'context [FieldDescriptor],
        buffer: NonNull<u8>,
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
        let buffer = unsafe { NonNull::new_unchecked(buffer) };

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
    pub fn buffer(&self) -> NonNull<u8> {
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
    ) -> Result<T::NonNullPtrs<'_>, ErasedSoaIntoValueError<Self>>
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
            let ptrs = T::ptrs_from_buffer(context, buffer.as_ptr(), capacity);
            let ptrs = T::ptrs_add_mut(context, ptrs, offset);
            let ptrs = T::ptrs_to_nonnull(context, ptrs);
            Ok(ptrs)
        }
    }

    #[inline]
    pub fn into_parts(self) -> (&'context [FieldDescriptor], NonNull<u8>, usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        (descriptors, buffer, capacity, offset)
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
    pub unsafe fn offset_from(self, origin: ErasedSoaNonNullPtrs<'_>) -> isize {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        assert_eq!(buffer, origin.buffer());
        assert_eq!(capacity, origin.capacity());
        assert_descriptors(descriptors, origin.field_descriptors());

        unsafe { (offset - origin.offset()).try_into().unwrap_unchecked() }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap(self, with: ErasedSoaNonNullPtrs<'_>, temp: &mut [MaybeUninit<u8>]) {
        let Self { descriptors, .. } = self;
        assert_descriptors(descriptors, with.field_descriptors());

        itertools::zip_eq(self, with).for_each(|(this, with)| unsafe { this.swap(with, temp) })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from(
        self,
        from: ErasedSoaNonNullPtrs<'_>,
        count: usize,
        temp: &mut [MaybeUninit<u8>],
    ) {
        let Self { descriptors, .. } = self;
        assert_descriptors(descriptors, from.field_descriptors());

        itertools::zip_eq(self, from)
            .for_each(|(this, from)| unsafe { this.copy_from(from, count, temp) })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_rev(
        self,
        from: ErasedSoaNonNullPtrs<'_>,
        count: usize,
        temp: &mut [MaybeUninit<u8>],
    ) {
        let Self { descriptors, .. } = self;
        assert_descriptors(descriptors, from.field_descriptors());

        #[inline]
        #[track_caller]
        fn rec(
            iter: &mut itertools::ZipEq<ErasedSoaNonNullPtrsIter<'_>, ErasedSoaNonNullPtrsIter<'_>>,
            count: usize,
            temp: &mut [MaybeUninit<u8>],
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
    pub unsafe fn copy_from_nonoverlapping(self, from: ErasedSoaNonNullPtrs<'_>, count: usize) {
        let Self { descriptors, .. } = self;
        assert_descriptors(descriptors, from.field_descriptors());

        itertools::zip_eq(self, from)
            .for_each(|(this, from)| unsafe { this.copy_from_nonoverlapping(from, count) })
    }
}

impl<'context> IntoIterator for ErasedSoaNonNullPtrs<'context> {
    type Item = ErasedFieldNonNullPtr;
    type IntoIter = ErasedSoaNonNullPtrsIter<'context>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        ErasedSoaNonNullPtrsIter {
            descriptors: descriptors.iter(),
            buffer,
            capacity,
            offset,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaNonNullPtrsIter<'context> {
    descriptors: slice::Iter<'context, FieldDescriptor>,
    buffer: NonNull<u8>,
    capacity: usize,
    offset: usize,
}

impl Debug for ErasedSoaNonNullPtrsIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_list().entries(entries).finish()
    }
}

impl ErasedSoaNonNullPtrsIter<'_> {
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_slice()
    }

    #[inline]
    pub fn buffer(&self) -> NonNull<u8> {
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

impl Iterator for ErasedSoaNonNullPtrsIter<'_> {
    type Item = ErasedFieldNonNullPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut buffer,
            capacity,
            offset,
        } = *self;

        let &desc = descriptors.next()?;
        let ptr_buffer = ptr::slice_from_raw_parts_mut(buffer.as_ptr(), desc.layout().size());
        let ptr_buffer = unsafe { NonNull::new_unchecked(ptr_buffer) };
        let ptr = unsafe { ErasedFieldNonNullPtr::new_unchecked(desc, ptr_buffer) };

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

impl ExactSizeIterator for ErasedSoaNonNullPtrsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }
}

impl FusedIterator for ErasedSoaNonNullPtrsIter<'_> {}
