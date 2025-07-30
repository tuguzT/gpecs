use core::{
    alloc::LayoutError,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ops::{Range, RangeBounds},
    ptr, slice,
};

use crate::{
    erased::{ErasedSoaSliceMutPtrs, ErasedSoaSlicePtrs, error::ErasedSoaIntoValueError},
    error::{check_layout, check_len},
    field::{ErasedFieldMutPtr, ErasedFieldSliceMut, field_slice_from_raw_parts_mut},
    soa::{
        slice::range,
        traits::{FieldDescriptor, Soa, buffer_layout},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSlicesMut<'a, D>
where
    D: ?Sized,
{
    buffer: *mut u8,
    capacity: usize,
    start: usize,
    end: usize,
    phantom: PhantomData<&'a mut [u8]>,
    descriptors: D,
}

impl<'a, D> ErasedSoaSlicesMut<'a, D> {
    #[inline]
    pub unsafe fn new_unchecked<R>(
        descriptors: D,
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
            phantom: PhantomData,
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
            ..
        } = self;
        (descriptors, buffer, capacity, start..end)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaSlicePtrs<D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
            ..
        } = self;
        unsafe { ErasedSoaSlicePtrs::new(descriptors, buffer, capacity, start..end) }
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaSliceMutPtrs<D> {
        let Self {
            descriptors,
            buffer,
            capacity,
            start,
            end,
            ..
        } = self;
        unsafe { ErasedSoaSliceMutPtrs::new(descriptors, buffer, capacity, start..end) }
    }
}

impl<'a, D> ErasedSoaSlicesMut<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    #[track_caller]
    pub fn new<R>(
        descriptors: D,
        buffer: &'a mut [u8],
        capacity: usize,
        range: R,
    ) -> Result<Self, LayoutError>
    where
        R: RangeBounds<usize>,
    {
        let layout = buffer_layout(descriptors.as_ref(), capacity)?;
        assert!(
            buffer.len() >= layout.size(),
            "buffer length ({buffer_len}) should be equal to or larger than expected layout size ({layout_size})",
            buffer_len = buffer.len(),
            layout_size = layout.size(),
        );

        let buffer = buffer.as_mut_ptr();
        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, range) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::SlicesMut<'_, 'a>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            start,
            end,
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
            let ptrs = T::ptrs_from_buffer(context, buffer, capacity);
            let ptrs = T::ptrs_add_mut(context, ptrs, start);
            let slices = T::slices_from_raw_parts_mut(context, ptrs, (start..end).len());
            let slice = T::slice_mut_ptrs_to_slices(context, slices);
            Ok(slice)
        }
    }
}

impl<D> ErasedSoaSlicesMut<'_, D>
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

impl<D> ErasedSoaSlicesMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
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
        let ErasedSoaSlicesMut {
            ref descriptors,
            buffer,
            capacity,
            start,
            end,
            phantom,
        } = *self;

        ErasedSoaSlicesMutIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            start,
            end,
            phantom,
        }
    }
}

impl<'a, D> IntoIterator for &'a mut ErasedSoaSlicesMut<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSliceMut<'a>;
    type IntoIter = ErasedSoaSlicesMutIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlicesMut {
            ref descriptors,
            buffer,
            capacity,
            start,
            end,
            phantom,
        } = *self;

        ErasedSoaSlicesMutIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            start,
            end,
            phantom,
        }
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
            buffer,
            capacity,
            start,
            end,
            phantom,
        } = self;

        ErasedSoaSlicesMutIter {
            descriptors: descriptors.into_iter(),
            buffer,
            capacity,
            start,
            end,
            phantom,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaSlicesMutIter<'a, D>
where
    D: ?Sized,
{
    buffer: *mut u8,
    capacity: usize,
    start: usize,
    end: usize,
    phantom: PhantomData<&'a mut [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaSlicesMutIter<'_, D>
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
            buffer,
            capacity,
            start,
            end,
            phantom,
            ..
        } = *self;

        let entries = ErasedSoaSlicesMutIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            start,
            end,
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
            ref mut buffer,
            capacity,
            start,
            end,
            ..
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let ptr_buffer = ptr::slice_from_raw_parts_mut(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, ptr_buffer) };

        let item = unsafe {
            let data = ptr.add(start);
            field_slice_from_raw_parts_mut(data, (start..end).len()).deref_mut()
        };
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
