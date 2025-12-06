use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::soa::field::FieldDescriptor;

use super::{
    ErasedFieldPtr,
    error::{ErasedFieldIntoValueError, ErasedFieldPtrError},
};

#[derive(Clone, Copy)]
pub struct ErasedFieldRef<'a> {
    inner: ErasedFieldPtr,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> ErasedFieldRef<'a> {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: &'a [u8]) -> Result<Self, ErasedFieldPtrError> {
        let field_ptr = ErasedFieldPtr::new(desc, buffer)?;
        let me = unsafe { Self::from_field_ptr(field_ptr) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a [u8]) -> Self {
        let field_ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) };
        unsafe { Self::from_field_ptr(field_ptr) }
    }

    #[inline]
    pub unsafe fn from_field_ptr(field_ptr: ErasedFieldPtr) -> Self {
        Self {
            inner: field_ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn try_into<T>(self) -> Result<&'a T, ErasedFieldIntoValueError<Self>> {
        let Self { inner, .. } = self;
        let into_self = |field_ptr| unsafe { Self::from_field_ptr(field_ptr) };
        let ptr = <*const T>::try_from(inner).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&T, ErasedFieldIntoValueError<&Self>> {
        let Self { inner, .. } = *self;
        let into_self = |_| self;
        let ptr = <*const T>::try_from(inner).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { inner, .. } = self;
        inner.descriptor()
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { inner, .. } = self;
        let buffer = inner.buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { inner, .. } = self;
        inner.as_ptr()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { inner, .. } = *self;
        inner
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [u8] {
        let (_, buffer) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a [u8]) {
        let Self { inner, .. } = self;
        let (desc, buffer) = inner.into_parts();
        let buffer = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        (desc, buffer)
    }
}

impl Debug for ErasedFieldRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.buffer();
        f.debug_struct("ErasedFieldRef")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl AsRef<[u8]> for ErasedFieldRef<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer()
    }
}

impl<'a, T> From<&'a T> for ErasedFieldRef<'a> {
    #[inline]
    fn from(r#ref: &'a T) -> Self {
        let field_ptr = ptr::from_ref(r#ref).into();
        unsafe { Self::from_field_ptr(field_ptr) }
    }
}
