use core::{
    fmt::{self, Debug},
    mem::ManuallyDrop,
    ptr, slice,
};

use crate::{
    aligned_bytes::{AlignedBoxedByteSlice, AlignedBytes, AlignedBytesFromLayout},
    error::{check_layout, check_len},
    field::error::{ErasedFieldError, ErasedFieldFromDescError},
    soa::traits::FieldDescriptor,
};

use super::{
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
    assert::check_into_layout, error::IntoValueError,
};

pub type BoxedErasedField = ErasedField<AlignedBoxedByteSlice>;

pub struct ErasedField<B>
where
    B: AlignedBytes + ?Sized,
{
    bytes: B,
}

impl<B> ErasedField<B>
where
    B: AlignedBytes,
{
    #[inline]
    pub fn new<T>(desc: FieldDescriptor, mut bytes: B, data: T) -> Result<Self, ErasedFieldError>
    where
        T: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let len = data.len();

        let layout = bytes.layout();
        let expected_layout = desc.layout();
        check_len(len, layout.size())?;
        check_len(len, expected_layout.size())?;
        check_layout(layout, expected_layout)?;

        bytes.copy_from(data)?;

        let me = Self { bytes };
        Ok(me)
    }

    #[inline]
    pub unsafe fn into<T>(self) -> Result<T, IntoValueError<Self>> {
        let desc = self.descriptor();
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { bytes } = me;

        let src = bytes.as_ptr().cast();
        Ok(unsafe { ptr::read(src) })
    }

    #[inline]
    pub fn into_bytes(self) -> B {
        let Self { bytes } = self;
        bytes
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, B) {
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
    pub fn from_desc<T>(desc: FieldDescriptor, data: T) -> Result<Self, ErasedFieldFromDescError<B>>
    where
        T: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let layout = desc.layout();
        check_len(data.len(), layout.size())?;

        let mut bytes = B::from_layout(layout).map_err(ErasedFieldFromDescError::FromDesc)?;
        bytes.copy_from(data)?;

        let me = Self { bytes };
        Ok(me)
    }

    #[inline]
    pub fn from<T>(value: T) -> Result<Self, B::Error> {
        let value = ManuallyDrop::new(value);
        let desc = FieldDescriptor::of::<T>();
        let data = ptr::from_ref(&value).cast();
        let data = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        match Self::from_desc(desc, data) {
            Ok(me) => Ok(me),
            Err(ErasedFieldFromDescError::FromDesc(err)) => Err(err),
            Err(ErasedFieldFromDescError::LenMismatch(err)) => unreachable!("{err}"),
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
    pub unsafe fn cast<T>(&self) -> Result<&T, IntoValueError<&Self>> {
        let desc = self.descriptor();
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { bytes } = me;

        let ptr = bytes.as_ptr().cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut T, IntoValueError<&mut Self>> {
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

        let data = bytes.as_ptr();
        let len = bytes.layout().size();
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { bytes, .. } = self;
        bytes.as_ptr()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        let Self { bytes } = self;

        let data = bytes.as_mut_ptr();
        let len = bytes.layout().size();
        unsafe { slice::from_raw_parts_mut(data, len) }
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
