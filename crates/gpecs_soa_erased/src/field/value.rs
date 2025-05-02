use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    mem::ManuallyDrop,
    ptr, slice,
};

use crate::{
    aligned_bytes::AlignedBytes, assert::check_same_len, error::LenMismatchError,
    soa::FieldDescriptor,
};

use super::{
    assert::check_layout, error::IntoValueError, ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef,
    ErasedFieldRefMut,
};

pub struct ErasedField {
    buffer: AlignedBytes,
}

impl ErasedField {
    #[inline]
    pub fn new<B>(desc: FieldDescriptor, buffer: B) -> Result<Self, LenMismatchError>
    where
        B: AsRef<[u8]>,
    {
        let buffer = buffer.as_ref();
        check_same_len(buffer.len(), desc.layout().size())?;

        let me = unsafe { Self::actual_new(desc, buffer) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked<B>(desc: FieldDescriptor, buffer: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(desc, buffer.as_ref()) }
    }

    #[inline]
    unsafe fn actual_new(desc: FieldDescriptor, buffer: &[u8]) -> Self {
        let mut bytes = AlignedBytes::new(desc.layout());
        unsafe {
            ptr::copy_nonoverlapping(buffer.as_ptr(), bytes.as_mut_ptr(), desc.layout().size());
        }
        Self { buffer: bytes }
    }

    #[inline]
    pub fn from<T>(value: T) -> Self {
        let value = ManuallyDrop::new(value);
        let desc = FieldDescriptor::of::<T>();
        let data = ptr::from_ref(&value).cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub unsafe fn into<T>(self) -> Result<T, IntoValueError<Self>> {
        let desc = self.descriptor();
        let me = check_layout::<T, _>(desc.layout(), self)?;
        let Self { buffer } = me;

        let src = buffer.as_ptr().cast();
        Ok(unsafe { ptr::read(src) })
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&T, IntoValueError<&Self>> {
        let desc = self.descriptor();
        let me = check_layout::<T, _>(desc.layout(), self)?;
        let Self { buffer } = me;

        let ptr = buffer.as_ptr().cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut T, IntoValueError<&mut Self>> {
        let desc = self.descriptor();
        let me = check_layout::<T, _>(desc.layout(), self)?;
        let Self { buffer } = me;

        let ptr = buffer.as_mut_ptr().cast();
        Ok(unsafe { &mut *ptr })
    }

    #[inline]
    pub fn as_field_ref(&self) -> ErasedFieldRef<'_> {
        let desc = self.descriptor();
        let buffer = self.buffer();
        unsafe { ErasedFieldRef::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let desc = self.descriptor();
        let buffer = ptr::from_ref(self.buffer());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ref_mut(&mut self) -> ErasedFieldRefMut<'_> {
        let desc = self.descriptor();
        let buffer = self.buffer_mut();
        unsafe { ErasedFieldRefMut::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_mut_ptr(&mut self) -> ErasedFieldMutPtr {
        let desc = self.descriptor();
        let buffer = ptr::from_mut(self.buffer_mut());
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { buffer } = self;
        FieldDescriptor::new(buffer.layout())
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { buffer } = self;
        unsafe { slice::from_raw_parts(buffer.as_ptr(), buffer.layout().size()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.as_ptr()
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { buffer } = self;
        unsafe { slice::from_raw_parts_mut(buffer.as_mut_ptr(), buffer.layout().size()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.as_mut_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> Box<[u8]> {
        let (_, buffer) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, Box<[u8]>) {
        let desc = self.descriptor();
        let Self { buffer } = self;

        let data = buffer.as_ptr();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        (desc, buffer.into())
    }
}

impl Debug for ErasedField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.buffer();
        f.debug_struct("ErasedField")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl AsRef<[u8]> for ErasedField {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer()
    }
}

impl AsMut<[u8]> for ErasedField {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.buffer_mut()
    }
}
