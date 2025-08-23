use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
    iter::FusedIterator,
    marker::PhantomData,
    ptr, slice,
};

use itertools::{EitherOrBoth::Both, Itertools};

use crate::{
    aligned_bytes::{AlignedBytes, AlignedBytesFromLayout},
    erased::{
        ErasedSoaRefs, ErasedSoaRefsMut,
        error::{
            ErasedSoaFromBytesFieldsDescriptorsError, ErasedSoaFromBytesValueError,
            ErasedSoaFromFieldsDescriptorsError, ErasedSoaFromValueError, ErasedSoaIntoValueError,
            IterOrFieldLenMismatchError,
        },
    },
    error::{LenMismatchError, check_layout, check_len},
    field::{ErasedField, error::ErasedFieldFromDescDataError},
    soa::{
        field::{BufferOffset, BufferOffsets, FieldDescriptor, buffer_layout, buffer_offsets},
        traits::{SoaRead, SoaWrite},
    },
};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoa = ErasedSoa<
    crate::aligned_bytes::AlignedUninitBoxedByteSlice,
    alloc::boxed::Box<[FieldDescriptor]>,
>;

pub struct ErasedSoa<B, D>
where
    B: ?Sized,
{
    descriptors: D,
    bytes: B,
}

impl<B, D> ErasedSoa<B, D>
where
    B: ?Sized,
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
        let expected_layout = buffer_layout(descriptors.as_ref(), 1)?;
        check_layout(layout, expected_layout)?;

        fill_bytes_with_fields(&mut bytes, fields, descriptors.as_ref())?;

        let me = Self { descriptors, bytes };
        Ok(me)
    }

    #[inline]
    pub unsafe fn into_value<T>(
        self,
        context: &T::Context,
    ) -> Result<T, ErasedSoaIntoValueError<Self>>
    where
        T: SoaRead,
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

        let layout = match T::buffer_layout(context, 1) {
            Ok(layout) => layout,
            Err(error) => return Err(ErasedSoaIntoValueError::new(self, error.into())),
        };
        if let Err(error) = check_len(layout.size(), bytes.layout().size()) {
            return Err(ErasedSoaIntoValueError::new(self, error.into()));
        }

        let Self { mut bytes, .. } = self;
        let value = unsafe {
            let src = T::ptrs_from_buffer(context, bytes.as_mut_ptr(), 1);
            T::read(context, T::ptrs_cast_const(context, src))
        };
        Ok(value)
    }
}

impl<B, D> ErasedSoa<B, D>
where
    B: AlignedBytes + ?Sized,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn as_refs(&self) -> ErasedSoaRefs<'_, &[FieldDescriptor]> {
        let Self { bytes, descriptors } = self;

        let descriptors = descriptors.as_ref();
        let buffer = bytes.as_ptr();
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, 1, 0) }
    }

    #[inline]
    pub fn as_refs_mut(&mut self) -> ErasedSoaRefsMut<'_, &[FieldDescriptor]> {
        let Self {
            ref mut bytes,
            ref descriptors,
        } = *self;

        let descriptors = descriptors.as_ref();
        let buffer = bytes.as_mut_ptr();
        unsafe { ErasedSoaRefsMut::new_unchecked(descriptors, buffer, 1, 0) }
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

        let layout = buffer_layout(descriptors.as_ref(), 1)?;
        let mut bytes = B::from_layout(layout).map_err(Error::FromLayout)?;
        fill_bytes_with_fields(&mut bytes, fields, descriptors.as_ref())?;

        let me = Self { descriptors, bytes };
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
    ) -> Result<Self, ErasedSoaFromBytesValueError>
    where
        T: SoaWrite,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| *desc.as_ref())
            .collect();

        let expected_layout = T::buffer_layout(context, 1)?;
        let layout = bytes.layout();
        check_layout(layout, expected_layout)?;

        unsafe {
            let dst = T::ptrs_from_buffer(context, bytes.as_mut_ptr(), 1);
            T::write(context, dst, value);
        }

        let me = Self { descriptors, bytes };
        Ok(me)
    }
}

impl<B, D> ErasedSoa<B, D>
where
    B: AlignedBytesFromLayout,
    D: FromIterator<FieldDescriptor>,
{
    #[inline]
    pub fn from_value<T>(context: &T::Context, value: T) -> Result<Self, ErasedSoaFromValueError<B>>
    where
        T: SoaWrite,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| *desc.as_ref())
            .collect();

        let layout = T::buffer_layout(context, 1)?;
        let mut bytes = B::from_layout(layout).map_err(ErasedSoaFromValueError::FromLayout)?;

        unsafe {
            let dst = T::ptrs_from_buffer(context, bytes.as_mut_ptr(), 1);
            T::write(context, dst, value);
        }

        let me = Self { descriptors, bytes };
        Ok(me)
    }
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
    B: AlignedBytes + ?Sized,
    D: AsRef<[FieldDescriptor]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = &self.as_refs().into_iter();
        f.debug_struct("ErasedSoa").field("fields", fields).finish()
    }
}

#[derive(Clone)]
pub struct ErasedSoaIntoFields<B, I, T>
where
    B: ?Sized,
{
    offsets: BufferOffsets<I>,
    phantom: PhantomData<T>,
    bytes: B,
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
    B: Debug + ?Sized,
    I: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { bytes, offsets, .. } = self;
        f.debug_struct("ErasedSoaIntoFields")
            .field("bytes", &bytes)
            .field("offsets", offsets)
            .finish()
    }
}

impl<B, I, T> Iterator for ErasedSoaIntoFields<B, I, T>
where
    B: AlignedBytes + ?Sized,
    I: Iterator,
    I::Item: AsRef<FieldDescriptor>,
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

        let Ok(item) = offsets.next()? else {
            unreachable!("buffer layout should be checked way earlier")
        };
        let BufferOffset {
            field_descriptor,
            offset,
            ..
        } = item;

        let len = field_descriptor.layout().size();
        let data = unsafe { bytes.as_ptr().add(offset) };
        let data = unsafe { slice::from_raw_parts(data, len) };

        let item = ErasedField::from_desc_data(field_descriptor, data);
        Some(item)
    }
}

impl<B, I, T> ExactSizeIterator for ErasedSoaIntoFields<B, I, T>
where
    B: AlignedBytes + ?Sized,
    I: ExactSizeIterator,
    I::Item: AsRef<FieldDescriptor>,
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
    B: AlignedBytes + ?Sized,
    I: FusedIterator,
    I::Item: AsRef<FieldDescriptor>,
    T: AlignedBytesFromLayout,
{
}

#[derive(Debug, Clone)]
enum FillBytesWithFieldsError {
    LenMismatch(IterOrFieldLenMismatchError),
    InvalidLayout(LayoutError),
}

impl From<IterOrFieldLenMismatchError> for FillBytesWithFieldsError {
    #[inline]
    fn from(value: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(value)
    }
}

impl From<LayoutError> for FillBytesWithFieldsError {
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl Display for FillBytesWithFieldsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(err) => Display::fmt(err, f),
            Self::InvalidLayout(err) => Display::fmt(err, f),
        }
    }
}

impl Error for FillBytesWithFieldsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(err) => Some(err),
            Self::InvalidLayout(err) => Some(err),
        }
    }
}

impl From<FillBytesWithFieldsError> for ErasedSoaFromBytesFieldsDescriptorsError {
    #[inline]
    fn from(value: FillBytesWithFieldsError) -> Self {
        match value {
            FillBytesWithFieldsError::LenMismatch(err) => Self::LenMismatch(err),
            FillBytesWithFieldsError::InvalidLayout(err) => Self::InvalidLayout(err),
        }
    }
}

impl<B> From<FillBytesWithFieldsError> for ErasedSoaFromFieldsDescriptorsError<B>
where
    B: AlignedBytesFromLayout,
{
    #[inline]
    fn from(value: FillBytesWithFieldsError) -> Self {
        match value {
            FillBytesWithFieldsError::LenMismatch(err) => Self::LenMismatch(err),
            FillBytesWithFieldsError::InvalidLayout(err) => Self::InvalidLayout(err),
        }
    }
}

fn fill_bytes_with_fields<B, I, F>(
    bytes: &mut B,
    fields: I,
    descriptors: &[FieldDescriptor],
) -> Result<(), FillBytesWithFieldsError>
where
    B: AlignedBytes + ?Sized,
    I: IntoIterator<Item = F>,
    F: AsRef<[u8]>,
{
    use IterOrFieldLenMismatchError::{FieldLenMismatch, IterLenMismatch};

    buffer_offsets(descriptors, 1)
        .zip_longest(fields)
        .enumerate()
        .try_for_each(|(field_index, item)| {
            let Both(item, src) = item else {
                let err = LenMismatchError::new(descriptors.len(), field_index);
                let err = IterLenMismatch(err).into();
                return Err(err);
            };
            let BufferOffset {
                field_descriptor,
                offset,
                ..
            } = item?;

            let src = src.as_ref();
            let len = field_descriptor.layout().size();
            check_len(src.len(), len).map_err(|error| FieldLenMismatch { error, field_index })?;

            let src = src.as_ptr();
            let dst = unsafe { bytes.as_mut_ptr().add(offset) };
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }

            Ok(())
        })
}
