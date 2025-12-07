use core::{fmt::Debug, ptr};

use crate::{error::check_align, soa::field::FieldDescriptor};

use super::{
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMut, ErasedFieldSlicePtr,
    assert::{check_into_layout, check_slice_buffer_len},
    error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldSliceMutPtr {
    ptr: ErasedFieldMutPtr,
    len: usize,
}

impl ErasedFieldSliceMutPtr {
    #[inline]
    #[track_caller]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        desc: FieldDescriptor,
        buffer: *mut [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = buffer.cast();
        check_align(ptr, desc.layout())?;
        check_slice_buffer_len(buffer.len(), desc.layout().size(), len)?;

        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        let me = unsafe { Self::from_ptr(ptr, len) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *mut [u8], len: usize) -> Self {
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        unsafe { Self::from_ptr(ptr, len) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldMutPtr, len: usize) -> Self {
        Self { ptr, len }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedFieldSlicePtr {
        let Self { ptr, len } = self;
        unsafe { ErasedFieldSlicePtr::from_ptr(ptr.cast_const(), len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a> {
        unsafe { ErasedFieldSlice::from_ptr(self.cast_const()) }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedFieldSliceMut<'a> {
        unsafe { ErasedFieldSliceMut::from_ptr(self) }
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
    pub fn as_mut_buffer(self) -> *mut [u8] {
        let Self { ptr, len } = self;
        let buffer = ptr.as_mut_buffer();
        ptr::slice_from_raw_parts_mut(buffer.cast(), len * buffer.len())
    }

    #[inline]
    pub fn as_ptr(self) -> *const u8 {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut u8 {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr()
    }

    #[inline]
    pub fn as_field_ptr(self) -> ErasedFieldPtr {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_field_mut_ptr(self) -> ErasedFieldMutPtr {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *mut [u8], usize) {
        let Self { ptr, len } = self;
        let (desc, buffer) = ptr.into_parts();
        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), len * buffer.len());
        (desc, buffer, len)
    }
}

impl<T> From<*mut [T]> for ErasedFieldSliceMutPtr {
    #[inline]
    fn from(ptr: *mut [T]) -> Self {
        let len = ptr.len();
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), desc.layout().size() * len);
        unsafe { Self::new_unchecked(desc, buffer, len) }
    }
}

impl<T> TryFrom<ErasedFieldSliceMutPtr> for *mut [T] {
    type Error = ErasedFieldIntoValueError<ErasedFieldSliceMutPtr>;

    #[inline]
    fn try_from(value: ErasedFieldSliceMutPtr) -> Result<Self, Self::Error> {
        let value = check_into_layout::<T, _>(value.descriptor().layout(), value)?;
        let ErasedFieldSliceMutPtr { ptr, len, .. } = value;

        let data = ptr.as_mut_ptr().cast();
        let slice = ptr::slice_from_raw_parts_mut(data, len);
        Ok(slice)
    }
}

#[inline]
pub fn field_slice_from_raw_parts_mut(
    data: ErasedFieldMutPtr,
    len: usize,
) -> ErasedFieldSliceMutPtr {
    unsafe { ErasedFieldSliceMutPtr::from_ptr(data, len) }
}
