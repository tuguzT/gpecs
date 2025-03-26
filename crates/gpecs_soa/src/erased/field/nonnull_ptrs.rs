use core::ptr::{self, NonNull};

use crate::traits::FieldDescriptor;

use super::assert::{assert_buffer_align, assert_layout, assert_value_buffer_len};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldNonNullPtr {
    desc: FieldDescriptor,
    ptr: NonNull<u8>,
}

impl ErasedFieldNonNullPtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: NonNull<[u8]>) -> Self {
        assert_value_buffer_len(buffer.len(), desc.layout().size());
        assert_buffer_align(buffer.as_ptr().cast(), desc.layout().align());

        let ptr = buffer.cast();
        Self { desc, ptr }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: NonNull<[u8]>) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer);
        }

        let ptr = buffer.cast();
        Self { desc, ptr }
    }

    #[inline]
    pub fn from<T>(ptr: NonNull<T>) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast(), desc.layout().size());
        let buffer = unsafe { NonNull::new_unchecked(ptr) };
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> NonNull<T> {
        let Self { desc, ptr } = self;
        assert_layout::<T>(desc.layout());

        ptr.cast()
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> NonNull<[u8]> {
        let Self { ptr, desc } = *self;
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast(), desc.layout().size());
        unsafe { NonNull::new_unchecked(ptr) }
    }

    #[inline]
    pub fn as_ptr(&self) -> NonNull<u8> {
        let Self { ptr: buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, NonNull<[u8]>) {
        let Self { desc, ptr } = self;
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast(), desc.layout().size());
        let buffer = unsafe { NonNull::new_unchecked(ptr) };
        (desc, buffer)
    }
}
