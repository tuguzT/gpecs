use core::{
    fmt::Debug,
    ptr::{self},
    slice,
};

use crate::{error::check_align, soa::field::FieldDescriptor};

use super::{
    ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
    assert::{check_into_layout, check_slice_buffer_len},
    error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldSlicePtr {
    desc: FieldDescriptor,
    ptr: *const u8,
    len: usize,
}

impl ErasedFieldSlicePtr {
    #[inline]
    #[track_caller]
    pub fn new(
        desc: FieldDescriptor,
        buffer: *const [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = buffer.cast();
        check_align(ptr, desc.layout())?;
        check_slice_buffer_len(buffer.len(), desc.layout().size(), len)?;

        Ok(Self { desc, ptr, len })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *const [u8], len: usize) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer, len).expect("incorrect inputs");
        }

        let ptr = buffer.cast();
        Self { desc, ptr, len }
    }

    #[inline]
    pub fn from<T>(ptr: *const [T]) -> Self {
        let len = ptr.len();
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), desc.layout().size() * len);
        unsafe { Self::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub fn into<T>(self) -> Result<*const [T], ErasedFieldIntoValueError<Self>> {
        let me = check_into_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, len, .. } = me;
        Ok(ptr::slice_from_raw_parts(ptr.cast(), len))
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldSliceMutPtr {
        let Self { desc, ptr, len } = self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast_mut(), desc.layout().size() * len);
        unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a> {
        let Self { desc, ptr, len } = self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
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
    pub fn buffer(&self) -> *const [u8] {
        let Self { desc, ptr, len } = *self;
        ptr::slice_from_raw_parts(ptr, len * desc.layout().size())
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *const [u8], usize) {
        let Self { desc, ptr, len } = self;
        let buffer = ptr::slice_from_raw_parts(ptr, len * desc.layout().size());
        (desc, buffer, len)
    }
}

#[inline]
pub fn field_slice_from_raw_parts(data: ErasedFieldPtr, len: usize) -> ErasedFieldSlicePtr {
    let (desc, data) = data.into_parts();
    let buffer = ptr::slice_from_raw_parts(data.cast(), len * desc.layout().size());
    unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) }
}
