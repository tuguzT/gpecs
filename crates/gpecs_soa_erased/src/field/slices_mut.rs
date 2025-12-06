use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::{self},
    slice,
};

use crate::soa::field::FieldDescriptor;

use super::{
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
    ErasedFieldSlicePtr,
    error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
};

pub struct ErasedFieldSliceMut<'a> {
    inner: ErasedFieldSliceMutPtr,
    phantom: PhantomData<&'a mut [u8]>,
}

impl<'a> ErasedFieldSliceMut<'a> {
    #[inline]
    #[track_caller]
    pub fn new(
        desc: FieldDescriptor,
        buffer: &'a mut [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = ErasedFieldSliceMutPtr::new(desc, buffer, len)?;
        let me = unsafe { Self::from_field_slice_mut_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a mut [u8], len: usize) -> Self {
        let ptr = unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) };
        unsafe { Self::from_field_slice_mut_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_field_slice_mut_ptr(ptr: ErasedFieldSliceMutPtr) -> Self {
        Self {
            inner: ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn try_into<T>(self) -> Result<&'a mut [T], ErasedFieldIntoValueError<Self>> {
        let Self { inner, .. } = self;
        let into_self = |ptr| unsafe { Self::from_field_slice_mut_ptr(ptr) };
        let buffer = <*mut [T]>::try_from(inner).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&[T], ErasedFieldIntoValueError<&Self>> {
        let Self { inner, .. } = *self;
        let into_self = |_| self;
        let buffer = <*mut [T]>::try_from(inner).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut [T], ErasedFieldIntoValueError<&mut Self>> {
        let Self { inner, .. } = *self;
        let into_self = |_| self;
        let buffer = <*mut [T]>::try_from(inner).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
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
        inner.as_ptr().cast_const()
    }

    #[inline]
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr {
        let Self { inner, .. } = self;
        inner.cast_const()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { inner, .. } = self;
        inner.as_field_ptr().cast_const()
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { inner, .. } = self;
        let buffer = inner.buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { inner, .. } = self;
        inner.as_ptr()
    }

    #[inline]
    pub fn as_field_slice_mut_ptr(&mut self) -> ErasedFieldSliceMutPtr {
        let Self { inner, .. } = *self;
        inner
    }

    #[inline]
    pub fn as_field_mut_ptr(&mut self) -> ErasedFieldMutPtr {
        let Self { inner, .. } = self;
        inner.as_field_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [u8] {
        let (_, buffer, _) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a mut [u8], usize) {
        let Self { inner, .. } = self;
        let (desc, buffer, len) = inner.into_parts();
        let buffer = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        (desc, buffer, len)
    }
}

impl Debug for ErasedFieldSliceMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.buffer();
        let len = &self.len();
        f.debug_struct("ErasedFieldSliceMut")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl AsRef<[u8]> for ErasedFieldSliceMut<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer()
    }
}

impl AsMut<[u8]> for ErasedFieldSliceMut<'_> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.buffer_mut()
    }
}

impl<'a, T> From<&'a mut [T]> for ErasedFieldSliceMut<'a> {
    #[inline]
    fn from(slice: &'a mut [T]) -> Self {
        let ptr = ptr::from_mut(slice).into();
        unsafe { Self::from_field_slice_mut_ptr(ptr) }
    }
}

impl<'a> From<ErasedFieldSliceMut<'a>> for ErasedFieldSlice<'a> {
    fn from(value: ErasedFieldSliceMut<'a>) -> Self {
        let (desc, buffer, len) = value.into_parts();
        unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
    }
}
