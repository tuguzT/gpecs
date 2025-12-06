use core::{
    fmt::Debug,
    ptr::{self},
    slice,
};

use crate::{error::check_align, soa::field::FieldDescriptor};

use super::{
    ErasedFieldMutPtr, ErasedFieldSlice, ErasedFieldSliceMut, ErasedFieldSlicePtr,
    assert::{check_into_layout, check_slice_buffer_len},
    error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldSliceMutPtr {
    desc: FieldDescriptor,
    ptr: *mut u8,
    len: usize,
}

impl ErasedFieldSliceMutPtr {
    #[inline]
    #[track_caller]
    pub fn new(
        desc: FieldDescriptor,
        buffer: *mut [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = buffer.cast();
        check_align(ptr, desc.layout())?;
        check_slice_buffer_len(buffer.len(), desc.layout().size(), len)?;

        Ok(Self { desc, ptr, len })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *mut [u8], len: usize) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer, len).expect("incorrect inputs");
        }

        let ptr = buffer.cast();
        Self { desc, ptr, len }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedFieldSlicePtr {
        let Self { desc, ptr, len } = self;
        let buffer = ptr::slice_from_raw_parts(ptr.cast_const(), desc.layout().size() * len);
        unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a> {
        let Self { desc, ptr, len } = self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedFieldSliceMut<'a> {
        let Self { desc, ptr, len } = self;
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSliceMut::new_unchecked(desc, buffer, len) }
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
    pub fn buffer(&self) -> *mut [u8] {
        let Self { desc, ptr, len } = *self;
        ptr::slice_from_raw_parts_mut(ptr, desc.layout().size() * len)
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldMutPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size());
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *mut [u8], usize) {
        let Self { desc, ptr, len } = self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size() * len);
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
        let ErasedFieldSliceMutPtr { desc, .. } = value;
        let value = check_into_layout::<T, _>(desc.layout(), value)?;

        let ErasedFieldSliceMutPtr { ptr, len, .. } = value;
        let slice = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
        Ok(slice)
    }
}

#[inline]
pub fn field_slice_from_raw_parts_mut(
    data: ErasedFieldMutPtr,
    len: usize,
) -> ErasedFieldSliceMutPtr {
    let (desc, data) = data.into_parts();
    let buffer = ptr::slice_from_raw_parts_mut(data.cast(), len * desc.layout().size());
    unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) }
}
