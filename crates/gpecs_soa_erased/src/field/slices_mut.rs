use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    field::{
        ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
        ErasedFieldSlicePtr,
        error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
    },
    fmt::DebugBytesUpperHex,
    soa::field::FieldDescriptor,
};

pub struct ErasedFieldSliceMut<'a> {
    ptr: ErasedFieldSliceMutPtr,
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
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a mut [u8], len: usize) -> Self {
        let ptr = unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) };
        unsafe { Self::from_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldSliceMutPtr) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn try_into<T>(self) -> Result<&'a mut [T], ErasedFieldIntoValueError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let buffer = <*mut [T]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&[T], ErasedFieldIntoValueError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = <*mut [T]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut [T], ErasedFieldIntoValueError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = <*mut [T]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
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
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { ptr, .. } = self;
        ptr.as_field_ptr()
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
    pub fn as_field_slice_mut_ptr(&mut self) -> ErasedFieldSliceMutPtr {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_mut_ptr(&mut self) -> ErasedFieldMutPtr {
        let Self { ptr, .. } = self;
        ptr.as_field_mut_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [u8] {
        let (_, buffer, _) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a mut [u8], usize) {
        let Self { ptr, .. } = self;
        let (desc, buffer, len) = ptr.into_parts();
        let buffer = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        (desc, buffer, len)
    }
}

impl Debug for ErasedFieldSliceMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &DebugBytesUpperHex(self.as_buffer());
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
        self.as_buffer()
    }
}

impl AsMut<[u8]> for ErasedFieldSliceMut<'_> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_buffer()
    }
}

impl<'a, T> From<&'a mut [T]> for ErasedFieldSliceMut<'a> {
    #[inline]
    fn from(slice: &'a mut [T]) -> Self {
        let ptr = ptr::from_mut(slice).into();
        unsafe { Self::from_ptr(ptr) }
    }
}

impl<'a> From<ErasedFieldSliceMut<'a>> for ErasedFieldSlice<'a> {
    fn from(value: ErasedFieldSliceMut<'a>) -> Self {
        let (desc, buffer, len) = value.into_parts();
        unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
    }
}
