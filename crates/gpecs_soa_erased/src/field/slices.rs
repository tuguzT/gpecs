use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::soa::field::FieldDescriptor;

use super::{
    ErasedFieldPtr, ErasedFieldSlicePtr,
    error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
};

#[derive(Clone, Copy)]
pub struct ErasedFieldSlice<'a> {
    inner: ErasedFieldSlicePtr,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> ErasedFieldSlice<'a> {
    #[inline]
    #[track_caller]
    pub fn new(
        desc: FieldDescriptor,
        buffer: &'a [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = ErasedFieldSlicePtr::new(desc, buffer, len)?;
        let me = unsafe { Self::from_field_slice_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a [u8], len: usize) -> Self {
        let ptr = unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) };
        unsafe { Self::from_field_slice_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_field_slice_ptr(ptr: ErasedFieldSlicePtr) -> Self {
        Self {
            inner: ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn try_into<T>(self) -> Result<&'a [T], ErasedFieldIntoValueError<Self>> {
        let Self { inner, .. } = self;
        let into_self = |ptr| unsafe { Self::from_field_slice_ptr(ptr) };
        let buffer = <*const [T]>::try_from(inner).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&[T], ErasedFieldIntoValueError<&Self>> {
        let Self { inner, .. } = *self;
        let into_self = |_| self;
        let buffer = <*const [T]>::try_from(inner).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr {
        let Self { inner, .. } = *self;
        inner
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { inner, .. } = self;
        inner.as_field_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [u8] {
        let (_, buffer, _) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a [u8], usize) {
        let Self { inner, .. } = self;
        let (desc, buffer, len) = inner.into_parts();
        let buffer = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        (desc, buffer, len)
    }
}

impl Debug for ErasedFieldSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.buffer();
        let len = &self.len();
        f.debug_struct("ErasedFieldSlice")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl AsRef<[u8]> for ErasedFieldSlice<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer()
    }
}

impl<'a, T> From<&'a [T]> for ErasedFieldSlice<'a> {
    #[inline]
    fn from(slice: &'a [T]) -> Self {
        let ptr = ptr::from_ref(slice).into();
        unsafe { Self::from_field_slice_ptr(ptr) }
    }
}
