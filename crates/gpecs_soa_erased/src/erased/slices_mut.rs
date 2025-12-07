use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaSliceMutPtrs, ErasedSoaSlicePtrs,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError, check_sufficient_len},
    },
    error::{check_layout, check_len},
    field::{ErasedFieldMutPtr, ErasedFieldSliceMut, field_slice_from_raw_parts_mut},
    soa::{
        field::{FieldDescriptor, buffer_layout},
        traits::{RawSoaContext, Soa},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSlicesMut<'a, D>
where
    D: ?Sized,
{
    ptr: *mut u8,
    capacity: usize,
    offset: usize,
    len: usize,
    phantom: PhantomData<&'a mut [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaSlicesMut<'_, D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *mut u8,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        Self {
            descriptors,
            ptr,
            capacity,
            offset,
            len,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *mut u8, usize, usize, usize) {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            len,
            ..
        } = self;
        (descriptors, ptr, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaSlicePtrs<D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            len,
            ..
        } = self;
        unsafe { ErasedSoaSlicePtrs::new_unchecked(descriptors, ptr, capacity, offset, len) }
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaSliceMutPtrs<D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            len,
            ..
        } = self;
        unsafe { ErasedSoaSliceMutPtrs::new_unchecked(descriptors, ptr, capacity, offset, len) }
    }
}

impl<'a, D> ErasedSoaSlicesMut<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    // TODO: check offset & len to be smaller than capacity
    #[inline]
    pub fn new<R>(
        descriptors: D,
        buffer: &'a mut [u8],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, ErasedSoaPtrsError> {
        let layout = buffer_layout(descriptors.as_ref(), capacity)?;
        check_sufficient_len(buffer.len(), layout.size())?;

        let ptr = buffer.as_mut_ptr();
        let me = unsafe { Self::new_unchecked(descriptors, ptr, capacity, offset, len) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::SlicesMut<'_, 'a>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            ref descriptors,
            ptr,
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

        let ptrs = unsafe { context.ptrs_from_buffer_mut(ptr, capacity) };
        let ptrs = unsafe { context.ptrs_add_mut(ptrs, offset) };
        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
        let slice = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        Ok(slice)
    }
}

impl<D> ErasedSoaSlicesMut<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr
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

impl<D> ErasedSoaSlicesMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicesMutIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
            len,
            ..
        } = *self;

        ErasedSoaSlicesMutIter {
            descriptors: descriptors.as_ref().iter(),
            ptr,
            capacity,
            offset,
            len,
            phantom: PhantomData,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaSlicesMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSliceMut<'a>;
    type IntoIter = ErasedSoaSlicesMutIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedSoaSlicesMut<'a, D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldSliceMut<'a>;
    type IntoIter = ErasedSoaSlicesMutIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            len,
            phantom,
        } = self;

        ErasedSoaSlicesMutIter {
            descriptors: descriptors.into_iter(),
            ptr,
            capacity,
            offset,
            len,
            phantom,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaSlicesMutIter<'a, D>
where
    D: ?Sized,
{
    ptr: *mut u8,
    capacity: usize,
    offset: usize,
    len: usize,
    phantom: PhantomData<&'a mut [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaSlicesMutIter<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr
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

impl<D> ErasedSoaSlicesMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D> Debug for ErasedSoaSlicesMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
            len,
            phantom,
            ..
        } = *self;

        let entries = ErasedSoaSlicesMutIter {
            descriptors: descriptors.as_ref().iter(),
            ptr,
            capacity,
            offset,
            len,
            phantom,
        };
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D> Iterator for ErasedSoaSlicesMutIter<'a, D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldSliceMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut ptr,
            capacity,
            offset,
            len,
            ..
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let buffer = ptr::slice_from_raw_parts_mut(*ptr, desc.layout().size());
        let field_ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };

        let data = unsafe { field_ptr.add(offset) };
        let item = unsafe { field_slice_from_raw_parts_mut(data, len).deref_mut() };
        *ptr = unsafe { field_ptr.add(capacity) }.as_mut_ptr();

        if let [desc, ..] = descriptors.as_ref() {
            *ptr = unsafe { ptr.add(ptr.align_offset(desc.layout().align())) };
        }
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { descriptors, .. } = self;
        descriptors.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedSoaSlicesMutIter<'_, D>
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

impl<D> FusedIterator for ErasedSoaSlicesMutIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
