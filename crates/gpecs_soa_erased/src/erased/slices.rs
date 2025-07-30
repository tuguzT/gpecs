use core::{
    alloc::LayoutError,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ops::{Range, RangeBounds},
    ptr, slice,
};

use crate::{
    erased::{ErasedSoaSlicePtrs, error::ErasedSoaIntoValueError},
    error::{check_layout, check_len},
    field::{ErasedFieldPtr, ErasedFieldSlice, field_slice_from_raw_parts},
    soa::{
        slice::range,
        traits::{FieldDescriptor, Soa, buffer_layout},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaSlices<'a, D>
where
    D: ?Sized,
{
    buffer: *const u8,
    capacity: usize,
    start: usize,
    end: usize,
    phantom: PhantomData<&'a [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaSlices<'_, D> {
    #[inline]
    pub unsafe fn new_unchecked<R>(
        descriptors: D,
        buffer: *const u8,
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
    pub fn into_parts(self) -> (D, *const u8, usize, Range<usize>) {
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
}

impl<'a, D> ErasedSoaSlices<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    #[track_caller]
    pub fn new<R>(
        descriptors: D,
        buffer: &'a [u8],
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

        let buffer = buffer.as_ptr();
        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, range) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::Slices<'_, 'a>, ErasedSoaIntoValueError<Self>>
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
            let ptrs = T::ptrs_from_buffer(context, buffer.cast_mut(), capacity);
            let ptrs = T::ptrs_add_mut(context, ptrs, start);
            let ptrs = T::ptrs_cast_const(context, ptrs);
            let slices = T::slices_from_raw_parts(context, ptrs, (start..end).len());
            let slices = T::slice_ptrs_to_slices(context, slices);
            Ok(slices)
        }
    }
}

impl<D> ErasedSoaSlices<'_, D>
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

impl<D> ErasedSoaSlices<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaSlices<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSlice<'a>;
    type IntoIter = ErasedSoaSlicesIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlices {
            ref descriptors,
            buffer,
            capacity,
            start,
            end,
            ..
        } = *self;

        ErasedSoaSlicesIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            start,
            end,
            phantom: PhantomData,
        }
    }
}

impl<'a, D> IntoIterator for &'a mut ErasedSoaSlices<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldSlice<'a>;
    type IntoIter = ErasedSoaSlicesIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaSlices {
            ref descriptors,
            buffer,
            capacity,
            start,
            end,
            ..
        } = *self;

        ErasedSoaSlicesIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            start,
            end,
            phantom: PhantomData,
        }
    }
}

impl<'a, D> IntoIterator for ErasedSoaSlices<'a, D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldSlice<'a>;
    type IntoIter = ErasedSoaSlicesIter<'a, D::IntoIter>;

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

        ErasedSoaSlicesIter {
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
pub struct ErasedSoaSlicesIter<'a, D>
where
    D: ?Sized,
{
    buffer: *const u8,
    capacity: usize,
    start: usize,
    end: usize,
    phantom: PhantomData<&'a [u8]>,
    descriptors: D,
}

impl<D> ErasedSoaSlicesIter<'_, D>
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
    pub fn range(&self) -> Range<usize> {
        let Self { start, end, .. } = *self;
        start..end
    }
}

impl<D> ErasedSoaSlicesIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D> Debug for ErasedSoaSlicesIter<'_, D>
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
            ..
        } = *self;

        let entries = ErasedSoaSlicesIter {
            descriptors: descriptors.as_ref().iter(),
            buffer,
            capacity,
            start,
            end,
            phantom: PhantomData,
        };
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D> Iterator for ErasedSoaSlicesIter<'a, D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldSlice<'a>;

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
        let ptr_buffer = ptr::slice_from_raw_parts(*buffer, desc.layout().size());
        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, ptr_buffer) };

        let item = unsafe {
            let data = ptr.add(start);
            field_slice_from_raw_parts(data, (start..end).len()).deref()
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

impl<D> ExactSizeIterator for ErasedSoaSlicesIter<'_, D>
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

impl<D> FusedIterator for ErasedSoaSlicesIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
