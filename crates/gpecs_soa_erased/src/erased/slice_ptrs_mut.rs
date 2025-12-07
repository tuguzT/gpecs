use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaMutPtrs, ErasedSoaPtrs, ErasedSoaSlicePtrs, ErasedSoaSlices, ErasedSoaSlicesMut,
        error::ErasedSoaIntoValueError,
    },
    error::{check_layout, check_len},
    field::{ErasedFieldMutPtr, ErasedFieldSliceMutPtr, field_slice_from_raw_parts_mut},
    soa::{
        field::FieldDescriptor,
        traits::{RawSoaContext, SliceMutPtrs, Soa},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSliceMutPtrs<D>
where
    D: ?Sized,
{
    ptr: *mut u8,
    capacity: usize,
    offset: usize,
    len: usize,
    descriptors: D,
}

impl<D> ErasedSoaSliceMutPtrs<D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *mut u8,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        Self {
            ptr,
            capacity,
            offset,
            len,
            descriptors,
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
        } = self;
        (descriptors, ptr, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            ..
        } = self;
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaMutPtrs<D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            ..
        } = self;
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSoaSlicePtrs<D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            len,
        } = self;

        let ptr = ptr.cast_const();
        unsafe { ErasedSoaSlicePtrs::new_unchecked(descriptors, ptr, capacity, offset, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaSlices<'a, D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            len,
        } = self;
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, ptr, capacity, offset, len) }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaSlicesMut<'a, D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            len,
        } = self;
        unsafe { ErasedSoaSlicesMut::new_unchecked(descriptors, ptr, capacity, offset, len) }
    }
}

impl<D> ErasedSoaSliceMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<SliceMutPtrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
            len,
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
        Ok(slices)
    }
}

impl<D> ErasedSoaSliceMutPtrs<D>
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

impl<D> ErasedSoaSliceMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSliceMutPtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
            len,
        } = *self;

        ErasedSoaSliceMutPtrsIter {
            descriptors: descriptors.as_ref().iter(),
            ptr,
            capacity,
            offset,
            len,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaSliceMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSliceMutPtr;
    type IntoIter = ErasedSoaSliceMutPtrsIter<slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> IntoIterator for ErasedSoaSliceMutPtrs<D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldSliceMutPtr;
    type IntoIter = ErasedSoaSliceMutPtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
            len,
        } = self;

        ErasedSoaSliceMutPtrsIter {
            descriptors: descriptors.into_iter(),
            ptr,
            capacity,
            offset,
            len,
        }
    }
}

#[inline]
pub fn slice_from_raw_parts_mut<D>(
    data: ErasedSoaMutPtrs<D>,
    len: usize,
) -> ErasedSoaSliceMutPtrs<D> {
    let (descriptors, ptr, capacity, offset) = data.into_parts();
    unsafe { ErasedSoaSliceMutPtrs::new_unchecked(descriptors, ptr, capacity, offset, len) }
}

#[derive(Clone)]
pub struct ErasedSoaSliceMutPtrsIter<D>
where
    D: ?Sized,
{
    ptr: *mut u8,
    capacity: usize,
    offset: usize,
    len: usize,
    descriptors: D,
}

impl<D> ErasedSoaSliceMutPtrsIter<D>
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
}

impl<D> ErasedSoaSliceMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D> Debug for ErasedSoaSliceMutPtrsIter<D>
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
        } = *self;

        let entries = ErasedSoaSliceMutPtrsIter {
            descriptors: descriptors.as_ref().iter(),
            ptr,
            capacity,
            offset,
            len,
        };
        f.debug_list().entries(entries).finish()
    }
}

impl<D> Iterator for ErasedSoaSliceMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldSliceMutPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut ptr,
            capacity,
            offset,
            len,
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let buffer = ptr::slice_from_raw_parts_mut(*ptr, desc.layout().size());
        let field_ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };

        let data = unsafe { field_ptr.add(offset) };
        let item = field_slice_from_raw_parts_mut(data, len);
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

impl<D> ExactSizeIterator for ErasedSoaSliceMutPtrsIter<D>
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

impl<D> FusedIterator for ErasedSoaSliceMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
