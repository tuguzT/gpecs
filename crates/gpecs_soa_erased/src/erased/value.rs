use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr, slice,
};
use itertools::{EitherOrBoth::Both, Itertools};

use crate::{
    aligned_bytes::{AlignedBytes, AlignedBytesFromLayout, AlignedUninitBoxedByteSlice},
    erased::{
        ErasedSoaRefs, ErasedSoaRefsMut,
        error::{
            ErasedSoaFromBytesFieldsDescriptorsError, ErasedSoaFromFieldsDescriptorsError,
            ErasedSoaIntoValueError, IterOrFieldLenMismatchError,
        },
    },
    error::{LayoutMismatchError, LenMismatchError, check_layout, check_len},
    field::{ErasedField, error::ErasedFieldFromDescDataError},
    soa::traits::{BufferOffsets, FieldDescriptor, Soa, buffer_layout, buffer_offsets},
};

pub type BoxedErasedSoa = ErasedSoa<AlignedUninitBoxedByteSlice, Box<[FieldDescriptor]>>;

pub struct ErasedSoa<B, D> {
    bytes: B,
    descriptors: D,
}

impl<B, D> ErasedSoa<B, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<B, D> ErasedSoa<B, D>
where
    B: AlignedBytes,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn from_bytes_fields_descriptors<I, F>(
        mut bytes: B,
        fields: I,
        descriptors: D,
    ) -> Result<Self, ErasedSoaFromBytesFieldsDescriptorsError>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<[u8]>,
    {
        let layout = bytes.layout();
        let expected_layout = buffer_layout(descriptors.as_ref(), 1)
            .expect("buffer layout size should not exceed `isize::MAX`");
        check_layout(layout, expected_layout)?;

        fill_bytes_with_fields(&mut bytes, fields, descriptors.as_ref())?;

        let me = Self { bytes, descriptors };
        Ok(me)
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedSoaRefs<'_, '_> {
        let Self { bytes, descriptors } = self;

        let descriptors = descriptors.as_ref();
        let buffer = bytes.as_ptr();
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, 1, 0) }
    }

    #[inline]
    pub fn as_refs_mut(&mut self) -> ErasedSoaRefsMut<'_, '_> {
        let Self {
            ref mut bytes,
            ref descriptors,
        } = *self;

        let descriptors = descriptors.as_ref();
        let buffer = bytes.as_mut_ptr();
        unsafe { ErasedSoaRefsMut::new_unchecked(descriptors, buffer, 1, 0) }
    }

    #[inline]
    pub unsafe fn into_value<T>(
        self,
        context: &T::Context,
    ) -> Result<T, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self {
            ref descriptors,
            ref bytes,
        } = self;
        let descriptors = descriptors.as_ref();

        let result = T::field_descriptors(context)
            .into_iter()
            .zip(descriptors)
            .try_fold(0, |len, (desc, self_desc)| {
                check_layout(self_desc.layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_len(len, descriptors.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(ErasedSoaIntoValueError::new(self, error));
        }

        let layout = T::buffer_layout(context, 1)
            .expect("buffer layout size should not exceed `isize::MAX`");
        if let Err(error) = check_len(layout.size(), bytes.layout().size()) {
            return Err(ErasedSoaIntoValueError::new(self, error.into()));
        }

        let Self { mut bytes, .. } = self;
        let value = unsafe {
            let src = T::ptrs_from_buffer(context, bytes.as_mut_ptr(), 1);
            T::ptrs_read(context, T::ptrs_cast_const(context, src))
        };
        Ok(value)
    }
}

impl<B, D> ErasedSoa<B, D>
where
    B: AlignedBytesFromLayout,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn from_fields_descriptors<I, F>(
        fields: I,
        descriptors: D,
    ) -> Result<Self, ErasedSoaFromFieldsDescriptorsError<B>>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<[u8]>,
    {
        use ErasedSoaFromFieldsDescriptorsError as Error;

        let layout = buffer_layout(descriptors.as_ref(), 1)
            .expect("buffer layout size should not exceed `isize::MAX`");

        let mut bytes = B::from_layout(layout).map_err(Error::FromLayout)?;
        fill_bytes_with_fields(&mut bytes, fields, descriptors.as_ref())?;

        let me = Self { bytes, descriptors };
        Ok(me)
    }
}

impl<B, D> ErasedSoa<B, D>
where
    B: AlignedBytes,
    D: FromIterator<FieldDescriptor>,
{
    #[inline]
    pub fn from_bytes_value<T>(
        mut bytes: B,
        context: &T::Context,
        value: T,
    ) -> Result<Self, LayoutMismatchError>
    where
        T: Soa,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| *desc.as_ref())
            .collect();

        let expected_layout = T::buffer_layout(context, 1)
            .expect("buffer layout size should not exceed `isize::MAX`");
        let layout = bytes.layout();
        check_layout(layout, expected_layout)?;

        unsafe {
            let dst = T::ptrs_from_buffer(context, bytes.as_mut_ptr(), 1);
            T::ptrs_write(context, dst, value);
        }

        let me = Self { bytes, descriptors };
        Ok(me)
    }
}

impl<B, D> ErasedSoa<B, D>
where
    B: AlignedBytesFromLayout,
    D: FromIterator<FieldDescriptor>,
{
    #[inline]
    pub fn from_value<T>(context: &T::Context, value: T) -> Result<Self, B::Error>
    where
        T: Soa,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| *desc.as_ref())
            .collect();

        let layout = T::buffer_layout(context, 1)
            .expect("buffer layout size should not exceed `isize::MAX`");
        let mut bytes = B::from_layout(layout)?;

        unsafe {
            let dst = T::ptrs_from_buffer(context, bytes.as_mut_ptr(), 1);
            T::ptrs_write(context, dst, value);
        }

        let me = Self { bytes, descriptors };
        Ok(me)
    }
}

#[derive(Clone)]
pub struct ErasedSoaIntoFields<B, I, T> {
    bytes: B,
    offsets: BufferOffsets<I>,
    phantom: PhantomData<T>,
}

impl<B, I, T> ErasedSoaIntoFields<B, I, T> {
    fn new(bytes: B, offsets: BufferOffsets<I>) -> Self {
        Self {
            bytes,
            offsets,
            phantom: PhantomData,
        }
    }
}

impl<B, I, T> Debug for ErasedSoaIntoFields<B, I, T>
where
    B: Debug,
    I: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { bytes, offsets, .. } = self;
        f.debug_struct("ErasedSoaIntoFields")
            .field("bytes", bytes)
            .field("offsets", offsets)
            .finish()
    }
}

impl<B, I, T> Iterator for ErasedSoaIntoFields<B, I, T>
where
    B: AlignedBytes,
    I: Iterator<Item: AsRef<FieldDescriptor>>,
    T: AlignedBytesFromLayout,
{
    type Item = Result<ErasedField<T>, ErasedFieldFromDescDataError<T>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref bytes,
            ref mut offsets,
            ..
        } = *self;

        let (desc, offset) = offsets.next()?.unwrap();
        let data = unsafe { bytes.as_ptr().add(offset) };
        let data = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        let item = ErasedField::from_desc_data(desc, data);
        Some(item)
    }
}

impl<B, I, T> ExactSizeIterator for ErasedSoaIntoFields<B, I, T>
where
    B: AlignedBytes,
    I: Iterator<Item: AsRef<FieldDescriptor>> + ExactSizeIterator,
    T: AlignedBytesFromLayout,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { offsets, .. } = self;
        offsets.len()
    }
}

impl<B, I, T> FusedIterator for ErasedSoaIntoFields<B, I, T>
where
    B: AlignedBytes,
    I: Iterator<Item: AsRef<FieldDescriptor>> + FusedIterator,
    T: AlignedBytesFromLayout,
{
}

impl<B, D> ErasedSoa<B, D>
where
    B: AlignedBytes,
    D: AsRef<[FieldDescriptor]> + IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    #[inline]
    pub fn into_fields<T>(self) -> ErasedSoaIntoFields<B, D::IntoIter, T>
    where
        T: AlignedBytesFromLayout,
    {
        let Self { bytes, descriptors } = self;

        let layout = buffer_layout(descriptors.as_ref(), 1)
            .expect("buffer layout size should not exceed `isize::MAX`");
        check_len(layout.size(), bytes.layout().size()).expect("buffer length should match");

        let offsets = buffer_offsets(descriptors, 1);
        ErasedSoaIntoFields::new(bytes, offsets)
    }
}

impl<B, D> Debug for ErasedSoa<B, D>
where
    B: AlignedBytes,
    D: AsRef<[FieldDescriptor]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = &self.as_refs().into_iter();
        f.debug_struct("ErasedSoa").field("fields", fields).finish()
    }
}

fn fill_bytes_with_fields<B, I, F>(
    bytes: &mut B,
    fields: I,
    descriptors: &[FieldDescriptor],
) -> Result<(), IterOrFieldLenMismatchError>
where
    B: AlignedBytes + ?Sized,
    I: IntoIterator<Item = F>,
    F: AsRef<[u8]>,
{
    use IterOrFieldLenMismatchError as Error;

    let mut field_index = 0;
    let descriptors_len = descriptors.len();
    let offsets = buffer_offsets(descriptors, 1).map(Result::unwrap);
    offsets.zip_longest(fields).try_for_each(|item| {
        let Both((desc, offset), src) = item else {
            let err = LenMismatchError::new(descriptors_len, field_index);
            let err = Error::IterLenMismatch(err);
            return Err(err);
        };

        let src = src.as_ref();
        let len = desc.layout().size();
        check_len(src.len(), len)
            .map_err(|error| Error::FieldLenMismatch { error, field_index })?;

        let src = src.as_ptr();
        let dst = unsafe { bytes.as_mut_ptr().add(offset) };
        unsafe {
            ptr::copy_nonoverlapping(src, dst, len);
        }

        field_index += 1;
        Ok(())
    })
}
