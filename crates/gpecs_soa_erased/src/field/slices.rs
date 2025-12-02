use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::{self},
    slice,
};

use crate::{error::check_align, soa::field::FieldDescriptor};

use super::{
    ErasedFieldPtr, ErasedFieldSlicePtr,
    assert::{check_into_layout, check_slice_buffer_len},
    error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
};

#[derive(Clone, Copy)]
pub struct ErasedFieldSlice<'a> {
    desc: FieldDescriptor,
    ptr: *const u8,
    len: usize,
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
        let ptr = buffer.as_ptr();
        check_align(buffer.as_ptr(), desc.layout())?;
        check_slice_buffer_len(buffer.len(), desc.layout().size(), len)?;

        Ok(Self {
            desc,
            ptr,
            len,
            phantom: PhantomData,
        })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a [u8], len: usize) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer, len).expect("incorrect inputs");
        }

        let ptr = buffer.as_ptr();
        Self {
            desc,
            ptr,
            len,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(ptr: &'a [T]) -> Self {
        let len = ptr.len();
        let desc = FieldDescriptor::of::<T>();
        let data = ptr.as_ptr().cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size() * len) };
        unsafe { Self::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub unsafe fn into<T>(self) -> Result<&'a [T], ErasedFieldIntoValueError<Self>> {
        let Self { desc, .. } = self;
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { ptr, len, .. } = me;

        let data = ptr.cast();
        Ok(unsafe { slice::from_raw_parts(data, len) })
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&[T], ErasedFieldIntoValueError<&Self>> {
        let Self { desc, .. } = *self;
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { ptr, len, .. } = *me;

        let data = ptr.cast();
        Ok(unsafe { slice::from_raw_parts(data, len) })
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { desc, ptr, len, .. } = *self;
        unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr {
        let Self { desc, ptr, len, .. } = *self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [u8] {
        let (_, buffer, _) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a [u8], usize) {
        let Self { desc, ptr, len, .. } = self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        (desc, buffer, len)
    }
}

impl Debug for ErasedFieldSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { desc, len, .. } = self;
        let buffer = &self.buffer();
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
