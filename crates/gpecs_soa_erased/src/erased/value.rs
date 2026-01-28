use core::{
    alloc::{Layout, LayoutError},
    error::Error,
    fmt::{self, Debug, Display},
    iter::FusedIterator,
    marker::PhantomData,
    mem::MaybeUninit,
    slice,
};

use itertools::{EitherOrBoth::Both, Itertools};

use crate::{
    erased::{
        ErasedSoaRefs, ErasedSoaRefsMut,
        error::{
            ErasedSoaFromFieldsDescriptorsError, ErasedSoaFromStorageFieldsDescriptorsError,
            ErasedSoaFromStorageValueError, ErasedSoaFromValueError, ErasedSoaIntoValueError,
            IterOrFieldLenMismatchError,
        },
    },
    error::{
        InsufficientAlignError, LenMismatchError, check_layout, check_len, check_sufficient_align,
    },
    field::{ErasedField, error::ErasedFieldFromDescDataError},
    soa::{
        field::{BufferOffset, BufferOffsets, FieldDescriptor, buffer_offsets},
        traits::{AllocSoa, AllocSoaContext, SoaRead, SoaWrite, WithFieldDescriptors},
    },
    storage::{AddressableUnit, AlignedStorage, AlignedStorageFromLayout},
    uninit::write_copy_of_slice,
};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoa =
    ErasedSoa<crate::storage::BoxedAlignedUninitStorage, alloc::boxed::Box<[FieldDescriptor]>, u8>;

pub struct ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<fn() -> A>,
    storage: T,
    descriptors: D,
}

impl<T, D, A> ErasedSoa<T, D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(storage: T, descriptors: D) -> Self {
        Self {
            phantom: PhantomData,
            storage,
            descriptors,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (T, D) {
        let Self {
            storage,
            descriptors,
            ..
        } = self;
        (storage, descriptors)
    }
}

impl<T, D, A> ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<T, D, A> ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn try_from_storage_fields_descriptors<I, F>(
        mut storage: T,
        fields: I,
        descriptors: D,
    ) -> Result<Self, ErasedSoaFromStorageFieldsDescriptorsError>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<[A]>,
    {
        let mut offsets = buffer_offsets(descriptors.as_ref(), 1);
        offsets.by_ref().try_for_each(|offset| {
            let desc = offset?.field_descriptor;
            check_sufficient_align(desc.layout(), Layout::new::<A>())
                .map_err(ErasedSoaFromStorageFieldsDescriptorsError::from)
        })?;

        let layout = storage.layout();
        let expected_layout = offsets.layout();
        check_layout(layout, expected_layout)?;

        write_copy_of_fields(storage.as_mut_uninit_slice(), fields, descriptors.as_ref())?;

        let me = unsafe { Self::new_unchecked(storage, descriptors) };
        Ok(me)
    }
}

impl<T, D> ErasedSoa<T, D, u8>
where
    T: AlignedStorage<u8>,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub unsafe fn try_into<V>(
        self,
        context: &V::Context,
    ) -> Result<V, ErasedSoaIntoValueError<Self>>
    where
        V: AllocSoa + SoaRead,
    {
        let Self {
            ref descriptors,
            ref storage,
            ..
        } = self;
        let descriptors = descriptors.as_ref();

        let result = context
            .field_descriptors()
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

        let layout = match context.buffer_layout(1) {
            Ok(layout) => layout,
            Err(error) => return Err(ErasedSoaIntoValueError::new(self, error.into())),
        };
        if let Err(error) = check_len(layout.size(), storage.layout().size()) {
            return Err(ErasedSoaIntoValueError::new(self, error.into()));
        }

        let Self { storage, .. } = self;
        let value = unsafe {
            let src = context.ptrs_from_buffer(storage.as_ptr(), 1);
            V::read(context, src)
        };
        Ok(value)
    }
}

impl<T, D, A> ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn as_fields(&self) -> ErasedSoaRefs<'_, &[FieldDescriptor], A> {
        let Self {
            ref storage,
            ref descriptors,
            ..
        } = *self;

        let descriptors = descriptors.as_ref();
        let buffer = storage.as_uninit_slice();
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, 1, 0) }
    }

    #[inline]
    pub fn as_mut_fields(&mut self) -> ErasedSoaRefsMut<'_, &[FieldDescriptor], A> {
        let Self {
            ref mut storage,
            ref descriptors,
            ..
        } = *self;

        let descriptors = descriptors.as_ref();
        let buffer = storage.as_mut_uninit_slice();
        unsafe { ErasedSoaRefsMut::new_unchecked(descriptors, buffer, 1, 0) }
    }
}

impl<T, D, A> ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn try_from_fields_descriptors<I, F>(
        fields: I,
        descriptors: D,
    ) -> Result<Self, ErasedSoaFromFieldsDescriptorsError<T, A>>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<[A]>,
    {
        use ErasedSoaFromFieldsDescriptorsError as Error;

        let mut offsets = buffer_offsets(descriptors.as_ref(), 1);
        offsets.by_ref().try_for_each(|offset| {
            let desc = offset?.field_descriptor;
            check_sufficient_align(desc.layout(), Layout::new::<A>()).map_err(Error::from)
        })?;

        let layout = offsets.layout();
        let mut storage = T::from_layout(layout).map_err(Error::FromLayout)?;

        write_copy_of_fields(storage.as_mut_uninit_slice(), fields, descriptors.as_ref())?;

        let me = unsafe { Self::new_unchecked(storage, descriptors) };
        Ok(me)
    }
}

impl<T, D> ErasedSoa<T, D, u8>
where
    T: AlignedStorage<u8>,
    D: FromIterator<FieldDescriptor>,
{
    #[inline]
    pub fn try_from_storage_value<V>(
        mut storage: T,
        context: &V::Context,
        value: V,
    ) -> Result<Self, ErasedSoaFromStorageValueError>
    where
        V: AllocSoa + SoaWrite,
    {
        let descriptors = context
            .field_descriptors()
            .into_iter()
            .map(|desc| *desc.as_ref())
            .collect();

        let expected_layout = context.buffer_layout(1)?;
        let layout = storage.layout();
        check_layout(layout, expected_layout)?;

        unsafe {
            let dst = context.ptrs_from_buffer_mut(storage.as_mut_ptr(), 1);
            V::write(context, dst, value);
        }

        let me = unsafe { Self::new_unchecked(storage, descriptors) };
        Ok(me)
    }
}

impl<T, D> ErasedSoa<T, D, u8>
where
    T: AlignedStorageFromLayout<u8>,
    D: FromIterator<FieldDescriptor>,
{
    #[inline]
    pub fn try_from<V>(
        context: &V::Context,
        value: V,
    ) -> Result<Self, ErasedSoaFromValueError<T, u8>>
    where
        V: AllocSoa + SoaWrite,
    {
        let descriptors = context
            .field_descriptors()
            .into_iter()
            .map(|desc| *desc.as_ref())
            .collect();

        let layout = context.buffer_layout(1)?;
        let mut storage = T::from_layout(layout).map_err(ErasedSoaFromValueError::FromLayout)?;

        unsafe {
            let dst = context.ptrs_from_buffer_mut(storage.as_mut_ptr(), 1);
            V::write(context, dst, value);
        }

        let me = unsafe { Self::new_unchecked(storage, descriptors) };
        Ok(me)
    }
}

impl<T, D, A> ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: AsRef<[FieldDescriptor]> + IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    #[inline]
    pub fn into_fields<F>(self) -> ErasedSoaIntoFields<T, D::IntoIter, F, A>
    where
        F: AlignedStorageFromLayout<A>,
    {
        let (storage, descriptors) = self.into_parts();
        let offsets = buffer_offsets(descriptors, 1);
        ErasedSoaIntoFields::new(storage, offsets)
    }
}

impl<T, D, A> Debug for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = &self.as_fields().into_iter();
        f.debug_struct("ErasedSoa").field("fields", fields).finish()
    }
}

pub struct ErasedSoaIntoFields<T, I, F, A>
where
    A: AddressableUnit,
    I: ?Sized,
{
    phantom: PhantomData<fn() -> (F, A)>,
    storage: T,
    offsets: BufferOffsets<I>,
}

impl<T, I, F, A> ErasedSoaIntoFields<T, I, F, A>
where
    A: AddressableUnit,
{
    fn new(storage: T, offsets: BufferOffsets<I>) -> Self {
        Self {
            phantom: PhantomData,
            storage,
            offsets,
        }
    }
}

impl<T, I, F, A> Debug for ErasedSoaIntoFields<T, I, F, A>
where
    A: AddressableUnit,
    T: Debug,
    I: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            storage, offsets, ..
        } = self;

        f.debug_struct("ErasedSoaIntoFields")
            .field("storage", storage)
            .field("offsets", &offsets)
            .finish()
    }
}

impl<T, I, F, A> Clone for ErasedSoaIntoFields<T, I, F, A>
where
    A: AddressableUnit,
    T: Clone,
    I: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            storage, offsets, ..
        } = self;

        Self::new(storage.clone(), offsets.clone())
    }
}

impl<T, I, F, A> Iterator for ErasedSoaIntoFields<T, I, F, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    I: Iterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
    F: AlignedStorageFromLayout<A>,
{
    type Item = Result<ErasedField<F, A>, ErasedFieldFromDescDataError<F, A>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref storage,
            ref mut offsets,
            ..
        } = *self;

        let BufferOffset {
            field_descriptor,
            offset,
            ..
        } = offsets
            .next()?
            .expect("buffer layout should have been checked way earlier");

        let offset = offset.div_ceil(size_of::<A>());
        let len = field_descriptor.layout().size().div_ceil(size_of::<A>());

        let data = unsafe { storage.as_ptr().add(offset) };
        let data = unsafe { slice::from_raw_parts(data, len) };

        let item = ErasedField::try_from_desc_data(field_descriptor, data);
        Some(item)
    }
}

impl<T, I, F, A> ExactSizeIterator for ErasedSoaIntoFields<T, I, F, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    I: ExactSizeIterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
    F: AlignedStorageFromLayout<A>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { offsets, .. } = self;
        offsets.len()
    }
}

impl<T, I, F, A> FusedIterator for ErasedSoaIntoFields<T, I, F, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    I: FusedIterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
    F: AlignedStorageFromLayout<A>,
{
}

#[derive(Debug, Clone)]
enum WriteCopyOfFieldsError {
    LenMismatch(IterOrFieldLenMismatchError),
    InvalidLayout(LayoutError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<IterOrFieldLenMismatchError> for WriteCopyOfFieldsError {
    #[inline]
    fn from(error: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutError> for WriteCopyOfFieldsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<InsufficientAlignError> for WriteCopyOfFieldsError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for WriteCopyOfFieldsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for WriteCopyOfFieldsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

impl From<WriteCopyOfFieldsError> for ErasedSoaFromStorageFieldsDescriptorsError {
    #[inline]
    fn from(error: WriteCopyOfFieldsError) -> Self {
        match error {
            WriteCopyOfFieldsError::LenMismatch(error) => Self::LenMismatch(error),
            WriteCopyOfFieldsError::InvalidLayout(error) => Self::InvalidLayout(error),
            WriteCopyOfFieldsError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

impl<T, A> From<WriteCopyOfFieldsError> for ErasedSoaFromFieldsDescriptorsError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    #[inline]
    fn from(error: WriteCopyOfFieldsError) -> Self {
        match error {
            WriteCopyOfFieldsError::LenMismatch(error) => Self::LenMismatch(error),
            WriteCopyOfFieldsError::InvalidLayout(error) => Self::InvalidLayout(error),
            WriteCopyOfFieldsError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

fn write_copy_of_fields<T, I, F>(
    dst: &mut [MaybeUninit<T>],
    fields: I,
    descriptors: &[FieldDescriptor],
) -> Result<(), WriteCopyOfFieldsError>
where
    T: Copy,
    I: IntoIterator<Item = F>,
    F: AsRef<[T]>,
{
    use IterOrFieldLenMismatchError::{FieldLenMismatch, IterLenMismatch};

    buffer_offsets(descriptors, 1)
        .zip_longest(fields)
        .enumerate()
        .try_for_each(|(field_index, item)| {
            let Both(offset, src) = item else {
                let error = LenMismatchError::new(descriptors.len(), field_index);
                let error = IterLenMismatch(error).into();
                return Err(error);
            };
            let BufferOffset {
                field_descriptor,
                offset,
                ..
            } = offset?;

            let layout = field_descriptor.layout();
            check_sufficient_align(layout, Layout::new::<T>())?;

            let offset = offset.div_ceil(size_of::<T>());
            let len = layout.size().div_ceil(size_of::<T>());
            write_copy_of_slice(&mut dst[offset..offset + len], src.as_ref())
                .map_err(|error| FieldLenMismatch { error, field_index })?;

            Ok(())
        })
}
