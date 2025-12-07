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
    ptr: ErasedFieldSlicePtr,
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
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn try_into<T>(self) -> Result<&'a [T], ErasedFieldIntoValueError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_field_slice_ptr(ptr) };
        let buffer = <*const [T]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&[T], ErasedFieldIntoValueError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = <*const [T]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { ptr, .. } = self;
        ptr.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[u8] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { ptr, .. } = self;
        ptr.as_field_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [u8] {
        let (_, buffer, _) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a [u8], usize) {
        let Self { ptr, .. } = self;
        let (desc, buffer, len) = ptr.into_parts();
        let buffer = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        (desc, buffer, len)
    }
}

impl Debug for ErasedFieldSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.as_buffer();
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
        self.as_buffer()
    }
}

impl<'a, T> From<&'a [T]> for ErasedFieldSlice<'a> {
    #[inline]
    fn from(slice: &'a [T]) -> Self {
        let ptr = ptr::from_ref(slice).into();
        unsafe { Self::from_field_slice_ptr(ptr) }
    }
}
