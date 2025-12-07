use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::soa::field::FieldDescriptor;

use super::{
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef,
    error::{ErasedFieldIntoValueError, ErasedFieldPtrError},
};

pub struct ErasedFieldRefMut<'a> {
    ptr: ErasedFieldMutPtr,
    phantom: PhantomData<&'a mut [u8]>,
}

impl<'a> ErasedFieldRefMut<'a> {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: &'a mut [u8]) -> Result<Self, ErasedFieldPtrError> {
        let ptr = ErasedFieldMutPtr::new(desc, buffer)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a mut [u8]) -> Self {
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        unsafe { Self::from_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldMutPtr) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn try_into<T>(self) -> Result<&'a mut T, ErasedFieldIntoValueError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let ptr = <*mut T>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_mut().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&T, ErasedFieldIntoValueError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = <*mut T>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut T, ErasedFieldIntoValueError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = <*mut T>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_mut().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[u8] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr().cast_const()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { ptr, .. } = *self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [u8] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr()
    }

    #[inline]
    pub fn as_field_mut_ptr(&mut self) -> ErasedFieldMutPtr {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [u8] {
        let (_, buffer) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a mut [u8]) {
        let Self { ptr, .. } = self;
        let (desc, buffer) = ptr.into_parts();
        let buffer = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        (desc, buffer)
    }
}

impl Debug for ErasedFieldRefMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.as_buffer();
        f.debug_struct("ErasedFieldRefMut")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl AsRef<[u8]> for ErasedFieldRefMut<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_buffer()
    }
}

impl AsMut<[u8]> for ErasedFieldRefMut<'_> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_buffer()
    }
}

impl<'a, T> From<&'a mut T> for ErasedFieldRefMut<'a> {
    #[inline]
    fn from(r#ref: &'a mut T) -> Self {
        let ptr = ptr::from_mut(r#ref).into();
        unsafe { Self::from_ptr(ptr) }
    }
}

impl<'a> From<ErasedFieldRefMut<'a>> for ErasedFieldRef<'a> {
    #[inline]
    fn from(value: ErasedFieldRefMut<'a>) -> Self {
        let ptr = value.as_field_ptr();
        unsafe { ErasedFieldRef::from_ptr(ptr) }
    }
}
