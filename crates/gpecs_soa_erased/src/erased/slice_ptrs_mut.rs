use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::{Range, RangeBounds},
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaMutPtrs, ErasedSoaPtrs, ErasedSoaSlicePtrs, ErasedSoaSlices, ErasedSoaSlicesMut,
        error::ErasedSoaIntoValueError,
    },
    error::{check_layout, check_len},
    field::{ErasedFieldMutPtr, ErasedFieldSliceMutPtr, field_slice_from_raw_parts_mut},
    soa::{field::FieldDescriptor, slice::range, traits::Soa},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSliceMutPtrs<D>
where
    D: ?Sized,
{
    buffer: *mut u8,
    capacity: usize,
    start: usize,
    end: usize,
    descriptors: D,
}

impl<D> ErasedSoaSliceMutPtrs<D> {
    #[inline]
    pub unsafe fn new<R>(descriptors: D, buffer: *mut u8, capacity: usize, range: R) -> Self
    where
        R: RangeBounds<usize>,
    {
        let Range { start, end } = self::range(range, ..capacity);
        Self {
            buffer,
            capacity,
            start,
            end,
            descriptors,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *mut u8, usize, Range<usize>) {
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
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            ..
        } = self;
        unsafe { ErasedSoaPtrs::new(descriptors, buffer, capacity, start) }
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaMutPtrs<D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            ..
        } = self;
        unsafe { ErasedSoaMutPtrs::new(descriptors, buffer, capacity, start) }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSoaSlicePtrs<D> {
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
    pub unsafe fn deref<'a>(self) -> ErasedSoaSlices<'a, D> {
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
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaSlicesMut<'a, D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
        } = self;
        unsafe { ErasedSoaSlicesMut::new_unchecked(descriptors, buffer, capacity, start..end) }
    }
}

impl<D> ErasedSoaSliceMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::SliceMutPtrs<'_>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            start,
            end,
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
            let ptrs = T::ptrs_from_buffer(context, buffer, capacity);
            let ptrs = T::ptrs_add_mut(context, ptrs, start);
            let slices = T::slices_from_raw_parts_mut(context, ptrs, (start..end).len());
            Ok(slices)
        }
    }
}

impl<D> ErasedSoaSliceMutPtrs<D>
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
            buffer,
            capacity,
            start,
            end,
        } = *self;

        ErasedSoaSliceMutPtrsIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            start,
            end,
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
            buffer,
            capacity,
            start,
            end,
        } = self;

        ErasedSoaSliceMutPtrsIter {
            descriptors: descriptors.into_iter(),
            buffer,
            capacity,
            start,
            end,
        }
    }
}

#[inline]
pub fn soa_slice_from_raw_parts_mut<D>(
    data: ErasedSoaMutPtrs<D>,
    len: usize,
) -> ErasedSoaSliceMutPtrs<D> {
    let (descriptors, buffer, capacity, start) = data.into_parts();
    let end = start.checked_add(len).unwrap();
    unsafe { ErasedSoaSliceMutPtrs::new(descriptors, buffer, capacity, start..end) }
}

#[derive(Clone)]
pub struct ErasedSoaSliceMutPtrsIter<D>
where
    D: ?Sized,
{
    buffer: *mut u8,
    capacity: usize,
    start: usize,
    end: usize,
    descriptors: D,
}

impl<D> ErasedSoaSliceMutPtrsIter<D>
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
    pub fn range(&self) -> Range<usize> {
        let Self { start, end, .. } = *self;
        start..end
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
            buffer,
            capacity,
            start,
            end,
        } = *self;

        let entries = ErasedSoaSliceMutPtrsIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            start,
            end,
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
            ref mut buffer,
            capacity,
            start,
            end,
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let ptr_buffer = ptr::slice_from_raw_parts_mut(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, ptr_buffer) };

        let item = field_slice_from_raw_parts_mut(unsafe { ptr.add(start) }, (start..end).len());
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
