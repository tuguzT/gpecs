use core::{
    alloc::LayoutError,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    erased::{ErasedSoaPtrs, error::ErasedSoaIntoValueError},
    error::{check_layout, check_len},
    field::{ErasedFieldPtr, ErasedFieldRef},
    soa::traits::{FieldDescriptor, Soa, buffer_layout},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaRefs<'a, D>
where
    D: ?Sized,
{
    buffer: *const u8,
    capacity: usize,
    offset: usize,
    phantom: PhantomData<&'a [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaRefs<'_, D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: *const u8,
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
    pub fn into_parts(self) -> (D, *const u8, usize, usize) {
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
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;
        unsafe { ErasedSoaPtrs::new(descriptors, buffer, capacity, offset) }
    }
}

impl<'a, D> ErasedSoaRefs<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    #[track_caller]
    pub fn new(
        descriptors: D,
        buffer: &'a [u8],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, LayoutError> {
        let layout = buffer_layout(descriptors.as_ref(), capacity)?;
        assert!(
            buffer.len() >= layout.size(),
            "buffer length ({buffer_len}) should be equal to or larger than expected layout size ({layout_size})",
            buffer_len = buffer.len(),
            layout_size = layout.size(),
        );

        let buffer = buffer.as_ptr();
        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::Refs<'_, 'a>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;
        let descriptors = descriptors.as_ref();

        let result = T::field_descriptors(context)
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

        unsafe {
            let ptrs = T::ptrs_from_buffer(context, buffer.cast_mut(), capacity);
            let ptrs = T::ptrs_add_mut(context, ptrs, offset);
            let ptrs = T::ptrs_cast_const(context, ptrs);
            let refs = T::ptrs_to_refs(context, ptrs);
            Ok(refs)
        }
    }
}

impl<D> ErasedSoaRefs<'_, D>
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

impl<D> ErasedSoaRefs<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaRefsIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;

        ErasedSoaRefsIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            offset,
            phantom: PhantomData,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaRefs<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldRef<'a>;
    type IntoIter = ErasedSoaRefsIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedSoaRefs<'a, D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldRef<'a>;
    type IntoIter = ErasedSoaRefsIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            phantom,
        } = self;

        ErasedSoaRefsIter {
            descriptors: descriptors.into_iter(),
            buffer,
            capacity,
            offset,
            phantom,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaRefsIter<'a, D>
where
    D: ?Sized,
{
    buffer: *const u8,
    capacity: usize,
    offset: usize,
    phantom: PhantomData<&'a [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaRefsIter<'_, D>
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

impl<D> ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D> Debug for ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;

        let entries = ErasedSoaRefsIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            offset,
            phantom: PhantomData,
        };
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D> Iterator for ErasedSoaRefsIter<'a, D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut buffer,
            capacity,
            offset,
            ..
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let ptr_buffer = ptr::slice_from_raw_parts(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, ptr_buffer) };

        let item = unsafe { ptr.add(offset).deref() };
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

impl<D> ExactSizeIterator for ErasedSoaRefsIter<'_, D>
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

impl<D> FusedIterator for ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
