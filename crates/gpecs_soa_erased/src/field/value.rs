use core::{
    fmt::{self, Debug},
    mem::{MaybeUninit, forget},
    ptr, slice,
};

use crate::{
    aligned_bytes::{AlignedBytes, AlignedBytesFromLayout, AlignedInitBytes},
    error::{LenMismatchError, check_layout, check_len},
    soa::field::FieldDescriptor,
};

#[cfg(feature = "alloc")]
use crate::aligned_bytes::AlignedUninitBoxedByteSlice;

use super::{
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
    assert::check_into_layout,
    error::{
        ErasedFieldFromBytesError, ErasedFieldFromDescDataError, ErasedFieldFromValueError,
        ErasedFieldIntoValueError,
    },
};

#[cfg(feature = "alloc")]
pub type BoxedErasedField = ErasedField<AlignedUninitBoxedByteSlice>;

pub struct ErasedField<B>
where
    B: ?Sized,
{
    bytes: AlignedInitBytes<B>,
}

impl<B> ErasedField<B>
where
    B: AlignedBytes,
{
    #[inline]
    pub fn from_bytes_desc_data<T>(
        mut bytes: B,
        desc: FieldDescriptor,
        data: T,
    ) -> Result<Self, ErasedFieldFromBytesError<B>>
    where
        T: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let len = data.len();

        let layout = bytes.layout();
        let expected_layout = desc.layout();
        if let Err(err) = check_len(len, layout.size()) {
            return Err(ErasedFieldFromBytesError::new(err.into(), bytes));
        }
        if let Err(err) = check_len(len, expected_layout.size()) {
            return Err(ErasedFieldFromBytesError::new(err.into(), bytes));
        }
        if let Err(err) = check_layout(layout, expected_layout) {
            return Err(ErasedFieldFromBytesError::new(err.into(), bytes));
        }

        if let Err(err) = init_bytes_from(bytes.as_uninit_bytes_mut(), data) {
            return Err(ErasedFieldFromBytesError::new(err.into(), bytes));
        }

        let bytes = unsafe { AlignedInitBytes::new_unchecked(bytes) };
        let me = Self { bytes };
        Ok(me)
    }

    #[inline]
    pub fn from_bytes_value<T>(
        bytes: B,
        value: T,
    ) -> Result<Self, ErasedFieldFromBytesError<(B, T)>> {
        let desc = FieldDescriptor::of::<T>();
        let data = ptr::from_ref(&value).cast();
        let data = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        match Self::from_bytes_desc_data(bytes, desc, data) {
            Ok(me) => {
                forget(value);
                Ok(me)
            }
            Err(err) => {
                let ErasedFieldFromBytesError { reason, bytes } = err;
                let err = ErasedFieldFromBytesError::new(reason, (bytes, value));
                Err(err)
            }
        }
    }

    #[inline]
    pub unsafe fn into_value<T>(self) -> Result<T, ErasedFieldIntoValueError<Self>> {
        let desc = self.descriptor();
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { bytes } = me;

        let src = bytes.as_ptr().cast();
        Ok(unsafe { ptr::read(src) })
    }

    #[inline]
    pub fn into_bytes(self) -> AlignedInitBytes<B> {
        let Self { bytes } = self;
        bytes
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, AlignedInitBytes<B>) {
        let desc = self.descriptor();
        let Self { bytes } = self;

        (desc, bytes)
    }
}

impl<B> ErasedField<B>
where
    B: AlignedBytesFromLayout,
{
    #[inline]
    pub fn from_desc_data<T>(
        desc: FieldDescriptor,
        data: T,
    ) -> Result<Self, ErasedFieldFromDescDataError<B>>
    where
        T: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let layout = desc.layout();
        check_len(data.len(), layout.size())?;

        let mut bytes = B::from_layout(layout).map_err(ErasedFieldFromDescDataError::FromLayout)?;
        init_bytes_from(bytes.as_uninit_bytes_mut(), data)?;

        let bytes = unsafe { AlignedInitBytes::new_unchecked(bytes) };
        let me = Self { bytes };
        Ok(me)
    }

    #[inline]
    pub fn from_value<T>(value: T) -> Result<Self, ErasedFieldFromValueError<B, T>> {
        let desc = FieldDescriptor::of::<T>();
        let data = ptr::from_ref(&value).cast();
        let data = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        match Self::from_desc_data(desc, data) {
            Ok(me) => {
                forget(value);
                Ok(me)
            }
            Err(ErasedFieldFromDescDataError::FromLayout(err)) => {
                let err = ErasedFieldFromValueError::new(err, value);
                Err(err)
            }
            Err(ErasedFieldFromDescDataError::LenMismatch(err)) => unreachable!("{err}"),
        }
    }
}

impl<B> ErasedField<B>
where
    B: AlignedBytes + ?Sized,
{
    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { bytes } = self;
        FieldDescriptor::new(bytes.layout())
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&T, ErasedFieldIntoValueError<&Self>> {
        let desc = self.descriptor();
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { bytes } = me;

        let ptr = bytes.as_ptr().cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut T, ErasedFieldIntoValueError<&mut Self>> {
        let desc = self.descriptor();
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { bytes } = me;

        let ptr = bytes.as_mut_ptr().cast();
        Ok(unsafe { &mut *ptr })
    }

    #[inline]
    pub fn as_field_ref(&self) -> ErasedFieldRef<'_> {
        let desc = self.descriptor();
        let buffer = self.as_slice();
        unsafe { ErasedFieldRef::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let desc = self.descriptor();
        let buffer = ptr::from_ref(self.as_slice());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ref_mut(&mut self) -> ErasedFieldRefMut<'_> {
        let desc = self.descriptor();
        let buffer = self.as_mut_slice();
        unsafe { ErasedFieldRefMut::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_mut_ptr(&mut self) -> ErasedFieldMutPtr {
        let desc = self.descriptor();
        let buffer = ptr::from_mut(self.as_mut_slice());
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        let Self { bytes } = self;
        bytes.as_slice()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { bytes, .. } = self;
        bytes.as_ptr()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        let Self { bytes } = self;
        bytes.as_mut_slice()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { bytes, .. } = self;
        bytes.as_mut_ptr()
    }
}

impl<B> Debug for ErasedField<B>
where
    B: AlignedBytes + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let data = &self.as_slice();
        f.debug_struct("ErasedField")
            .field("desc", desc)
            .field("data", data)
            .finish()
    }
}

impl<B> AsRef<[u8]> for ErasedField<B>
where
    B: AlignedBytes + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<B> AsMut<[u8]> for ErasedField<B>
where
    B: AlignedBytes + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

#[inline]
fn init_bytes_from(dst: &mut [MaybeUninit<u8>], src: &[u8]) -> Result<(), LenMismatchError> {
    let expected = dst.len();
    let len = src.len();
    check_len(len, expected)?;

    let src = unsafe { slice::from_raw_parts(src.as_ptr().cast(), len) };
    dst.copy_from_slice(src);

    Ok(())
}
