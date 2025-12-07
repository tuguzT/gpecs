use core::{fmt::Debug, ptr};

use crate::{error::check_align, soa::field::FieldDescriptor};

use super::{
    ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
    assert::{check_into_layout, check_slice_buffer_len},
    error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldSlicePtr {
    ptr: ErasedFieldPtr,
    len: usize,
}

impl ErasedFieldSlicePtr {
    #[inline]
    #[track_caller]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        desc: FieldDescriptor,
        buffer: *const [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = buffer.cast();
        check_align(ptr, desc.layout())?;
        check_slice_buffer_len(buffer.len(), desc.layout().size(), len)?;

        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) };
        let me = unsafe { Self::from_field_ptr(ptr, len) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *const [u8], len: usize) -> Self {
        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) };
        unsafe { Self::from_field_ptr(ptr, len) }
    }

    #[inline]
    pub unsafe fn from_field_ptr(ptr: ErasedFieldPtr, len: usize) -> Self {
        Self { ptr, len }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldSliceMutPtr {
        let Self { ptr, len } = self;
        unsafe { ErasedFieldSliceMutPtr::from_field_mut_ptr(ptr.cast_mut(), len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a> {
        unsafe { ErasedFieldSlice::from_field_slice_ptr(self) }
    }

    #[inline]
    pub fn len(self) -> usize {
        let Self { len, .. } = self;
        len
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [u8] {
        let Self { ptr, len } = self;
        let buffer = ptr.as_buffer();
        ptr::slice_from_raw_parts(buffer.cast(), len * buffer.len())
    }

    #[inline]
    pub fn as_ptr(self) -> *const u8 {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_field_ptr(self) -> ErasedFieldPtr {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *const [u8], usize) {
        let Self { ptr, len } = self;
        let (desc, buffer) = ptr.into_parts();
        let buffer = ptr::slice_from_raw_parts(buffer.cast(), len * buffer.len());
        (desc, buffer, len)
    }
}

impl<T> From<*const [T]> for ErasedFieldSlicePtr {
    #[inline]
    fn from(ptr: *const [T]) -> Self {
        let len = ptr.len();
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), desc.layout().size() * len);
        unsafe { Self::new_unchecked(desc, buffer, len) }
    }
}

impl<T> TryFrom<ErasedFieldSlicePtr> for *const [T] {
    type Error = ErasedFieldIntoValueError<ErasedFieldSlicePtr>;

    #[inline]
    fn try_from(value: ErasedFieldSlicePtr) -> Result<Self, Self::Error> {
        let value = check_into_layout::<T, _>(value.descriptor().layout(), value)?;
        let ErasedFieldSlicePtr { ptr, len, .. } = value;

        let data = ptr.as_ptr().cast();
        let slice = ptr::slice_from_raw_parts(data, len);
        Ok(slice)
    }
}

#[inline]
pub fn field_slice_from_raw_parts(data: ErasedFieldPtr, len: usize) -> ErasedFieldSlicePtr {
    unsafe { ErasedFieldSlicePtr::from_field_ptr(data, len) }
}
