use core::{
    alloc::LayoutError,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    erased::{ErasedSoaMutPtrs, ErasedSoaPtrs, error::ErasedSoaIntoValueError},
    error::{check_layout, check_len},
    field::{ErasedFieldMutPtr, ErasedFieldRefMut},
    soa::{
        field::{FieldDescriptor, buffer_layout},
        traits::{Soa, SoaContext},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaRefsMut<'a, D>
where
    D: ?Sized,
{
    buffer: *mut u8,
    capacity: usize,
    offset: usize,
    phantom: PhantomData<&'a mut [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaRefsMut<'_, D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
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
    pub fn into_parts(self) -> (D, *mut u8, usize, usize) {
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

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaMutPtrs<D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;
        unsafe { ErasedSoaMutPtrs::new(descriptors, buffer, capacity, offset) }
    }
}

impl<'a, D> ErasedSoaRefsMut<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    #[track_caller]
    pub fn new(
        descriptors: D,
        buffer: &'a mut [u8],
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

        let buffer = buffer.as_mut_ptr();
        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::RefsMut<'_, 'a>, ErasedSoaIntoValueError<Self>>
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

        unsafe {
            let ptrs = context.ptrs_from_buffer_mut(buffer, capacity);
            let ptrs = context.ptrs_add_mut(ptrs, offset);
            let refs = T::ptrs_to_refs_mut(context, ptrs);
            Ok(refs)
        }
    }
}

impl<D> ErasedSoaRefsMut<'_, D>
where
    D: ?Sized,
{
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

impl<D> ErasedSoaRefsMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaRefsMutIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;

        ErasedSoaRefsMutIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            offset,
            phantom: PhantomData,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaRefsMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldRefMut<'a>;
    type IntoIter = ErasedSoaRefsMutIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedSoaRefsMut<'a, D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldRefMut<'a>;
    type IntoIter = ErasedSoaRefsMutIter<'a, D::IntoIter>;

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
            descriptors: descriptors.into_iter(),
            buffer,
            capacity,
            offset,
            phantom,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaRefsMutIter<'a, D>
where
    D: ?Sized,
{
    buffer: *mut u8,
    capacity: usize,
    offset: usize,
    phantom: PhantomData<&'a mut [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaRefsMutIter<'_, D>
where
    D: ?Sized,
{
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

impl<D> ErasedSoaRefsMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D> Debug for ErasedSoaRefsMutIter<'_, D>
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

        let entries = ErasedSoaRefsMutIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            offset,
            phantom: PhantomData,
        };
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D> Iterator for ErasedSoaRefsMutIter<'a, D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
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

        let &desc = descriptors.next()?.as_ref();
        let ptr_buffer = ptr::slice_from_raw_parts_mut(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, ptr_buffer) };

        let item = unsafe { ptr.add(offset).deref_mut() };
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

impl<D> ExactSizeIterator for ErasedSoaRefsMutIter<'_, D>
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

impl<D> FusedIterator for ErasedSoaRefsMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
