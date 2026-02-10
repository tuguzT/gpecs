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
        CovariantFieldDescriptors, ErasedSoaRefs, ErasedSoaRefsMut,
        assert::check_into_value,
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
    slice_item_ptr::SliceItemPtrs,
    soa::{
        field::{
            BufferOffset, BufferOffsets, FieldDescriptor, FieldDescriptors, FieldDescriptorsIter,
            FieldDescriptorsOwned, buffer_offsets,
        },
        traits::{AllocSoa, AllocSoaContext, SoaRead, SoaWrite},
    },
    storage::{AlignedStorage, AlignedStorageFromLayout},
    uninit::write_copy_of_slice,
};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoa<P> =
    ErasedSoa<crate::storage::BoxedAlignedUninitStorage, alloc::boxed::Box<[FieldDescriptor]>, P>;

pub struct ErasedSoa<T, D, P>
where
    D: ?Sized,
{
    phantom: PhantomData<P>,
    storage: T,
    descriptors: D,
}

impl<T, D, P> ErasedSoa<T, D, P> {
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

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub fn try_from_storage_fields_descriptors<I, F>(
        mut storage: T,
        fields: I,
        descriptors: D,
    ) -> Result<Self, ErasedSoaFromStorageFieldsDescriptorsError>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<[T::Item]>,
    {
        let mut offsets = buffer_offsets(descriptors.field_descriptors(), 1);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<T::Item>())
                .map_err(ErasedSoaFromStorageFieldsDescriptorsError::from)
        })?;

        let layout = storage.layout();
        let expected_layout = offsets.into_layout();
        check_layout(layout, expected_layout)?;

        write_copy_of_fields(
            storage.as_mut_uninit_slice(),
            fields,
            descriptors.field_descriptors(),
        )?;

        let me = unsafe { Self::new_unchecked(storage, descriptors) };
        Ok(me)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage<Item = u8>,
    D: FieldDescriptorsOwned,
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

        if let Err(error) =
            check_into_value(descriptors.field_descriptors(), context.field_descriptors())
        {
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

impl<'a, T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: FieldDescriptors<'a> + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn as_fields(&'a self) -> ErasedSoaRefs<'a, D::Output, P::Const> {
        let Self {
            ref storage,
            ref descriptors,
            ..
        } = *self;

        let descriptors = descriptors.field_descriptors();
        let buffer = storage.as_uninit_slice();
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, 1, 0) }
    }

    #[inline]
    pub fn as_mut_fields(&'a mut self) -> ErasedSoaRefsMut<'a, D::Output, P::Mut> {
        let Self {
            ref mut storage,
            ref descriptors,
            ..
        } = *self;

        let descriptors = descriptors.field_descriptors();
        let buffer = storage.as_mut_uninit_slice();
        unsafe { ErasedSoaRefsMut::new_unchecked(descriptors, buffer, 1, 0) }
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorageFromLayout,
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub fn try_from_fields_descriptors<I, F>(
        fields: I,
        descriptors: D,
    ) -> Result<Self, ErasedSoaFromFieldsDescriptorsError<T::Error>>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<[T::Item]>,
    {
        use ErasedSoaFromFieldsDescriptorsError as Error;

        let mut offsets = buffer_offsets(descriptors.field_descriptors(), 1);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<T::Item>())
                .map_err(Error::from)
        })?;

        let layout = offsets.into_layout();
        let mut storage = T::from_layout(layout).map_err(Error::FromLayout)?;

        write_copy_of_fields(
            storage.as_mut_uninit_slice(),
            fields,
            descriptors.field_descriptors(),
        )?;

        let me = unsafe { Self::new_unchecked(storage, descriptors) };
        Ok(me)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage<Item = u8>,
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

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorageFromLayout<Item = u8>,
    D: FromIterator<FieldDescriptor>,
{
    #[inline]
    pub fn try_from<V>(
        context: &V::Context,
        value: V,
    ) -> Result<Self, ErasedSoaFromValueError<T::Error>>
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

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    pub fn into_fields<F>(self) -> ErasedSoaIntoFields<T, D::IntoIter, F, P>
    where
        F: AlignedStorageFromLayout<Item = T::Item>,
    {
        let (storage, descriptors) = self.into_parts();
        let offsets = buffer_offsets(descriptors, 1);
        ErasedSoaIntoFields::new(storage, offsets)
    }
}

impl<T, D, P> Debug for ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: FieldDescriptorsOwned + ?Sized,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
    for<'a, 'b> FieldDescriptorsIter<'a, D>: FieldDescriptors<'b>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = &self.as_fields().into_iter();
        f.debug_struct("ErasedSoa").field("fields", fields).finish()
    }
}

impl<'a, T, D, P> FieldDescriptors<'a> for ErasedSoa<T, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { descriptors, .. } = self;
        descriptors.field_descriptors()
    }
}

impl<T, D, P> CovariantFieldDescriptors for ErasedSoa<T, D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedSoaIntoFields<T, I, F, P>
where
    I: ?Sized,
{
    phantom: PhantomData<fn() -> (F, P)>,
    storage: T,
    offsets: BufferOffsets<I>,
}

impl<T, I, F, P> ErasedSoaIntoFields<T, I, F, P> {
    fn new(storage: T, offsets: BufferOffsets<I>) -> Self {
        Self {
            phantom: PhantomData,
            storage,
            offsets,
        }
    }
}

impl<T, I, F, P> Debug for ErasedSoaIntoFields<T, I, F, P>
where
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

impl<T, I, F, P> Clone for ErasedSoaIntoFields<T, I, F, P>
where
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

impl<T, I, F, P> Iterator for ErasedSoaIntoFields<T, I, F, P>
where
    T: AlignedStorage,
    I: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    F: AlignedStorageFromLayout<Item = T::Item>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    type Item = Result<ErasedField<F, P>, ErasedFieldFromDescDataError<F::Error>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref storage,
            ref mut offsets,
            ..
        } = *self;

        let BufferOffset { desc, offset, .. } = unsafe { offsets.next()?.unwrap_unchecked() };

        let offset = offset.div_ceil(size_of::<T::Item>());
        let len = desc.layout().size().div_ceil(size_of::<T::Item>());

        let data = unsafe { storage.as_ptr().add(offset) };
        let data = unsafe { slice::from_raw_parts(data, len) };

        let item = ErasedField::try_from_desc_data(desc, data);
        Some(item)
    }
}

impl<T, I, F, P> ExactSizeIterator for ErasedSoaIntoFields<T, I, F, P>
where
    T: AlignedStorage,
    I: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    F: AlignedStorageFromLayout<Item = T::Item>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { offsets, .. } = self;
        offsets.len()
    }
}

impl<T, I, F, P> FusedIterator for ErasedSoaIntoFields<T, I, F, P>
where
    T: AlignedStorage,
    I: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    F: AlignedStorageFromLayout<Item = T::Item>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
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

impl<T> From<WriteCopyOfFieldsError> for ErasedSoaFromFieldsDescriptorsError<T> {
    #[inline]
    fn from(error: WriteCopyOfFieldsError) -> Self {
        match error {
            WriteCopyOfFieldsError::LenMismatch(error) => Self::LenMismatch(error),
            WriteCopyOfFieldsError::InvalidLayout(error) => Self::InvalidLayout(error),
            WriteCopyOfFieldsError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

fn write_copy_of_fields<T, F, D>(
    dst: &mut [MaybeUninit<T>],
    fields: F,
    descriptors: D,
) -> Result<(), WriteCopyOfFieldsError>
where
    T: Copy,
    F: IntoIterator<Item: AsRef<[T]>>,
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    use IterOrFieldLenMismatchError::{FieldLenMismatch, IterLenMismatch};

    let mut descriptors = descriptors.into_iter();
    for (field_index, item) in buffer_offsets(&mut descriptors, 1)
        .zip_longest(fields)
        .enumerate()
    {
        let Both(offset, src) = item else {
            let descriptors_count = field_index + descriptors.count();
            let error = unsafe { LenMismatchError::new_unchecked(descriptors_count, field_index) };
            let error = IterLenMismatch(error).into();
            return Err(error);
        };
        let BufferOffset { desc, offset, .. } = offset?;

        let layout = desc.layout();
        check_sufficient_align(layout, Layout::new::<T>())?;

        let offset = offset.div_ceil(size_of::<T>());
        let len = layout.size().div_ceil(size_of::<T>());
        write_copy_of_slice(&mut dst[offset..offset + len], src.as_ref())
            .map_err(|error| FieldLenMismatch { error, field_index })?;
    }
    Ok(())
}
