use core::ptr::{self, NonNull};

use crate::traits::FieldDescriptor;

use super::assert::{assert_buffer_align, assert_layout, assert_value_buffer_len};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldNonNullPtr {
    desc: FieldDescriptor,
    buffer: NonNull<[u8]>,
}

impl ErasedFieldNonNullPtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: NonNull<[u8]>) -> Self {
        assert_value_buffer_len(buffer.len(), desc.layout().size());
        assert_buffer_align(buffer.as_ptr().cast(), desc.layout().align());

        Self { desc, buffer }
    }

    #[inline]
    pub fn from<T>(ptr: NonNull<T>) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast(), desc.layout().size());
        let buffer = NonNull::new(ptr).expect("input pointer should be nonnull");
        Self::new(desc, buffer)
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> NonNull<T> {
        let Self { desc, buffer } = self;
        assert_layout::<T>(desc.layout());

        buffer.cast()
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> NonNull<[u8]> {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> NonNull<u8> {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, NonNull<[u8]>) {
        let Self { desc, buffer } = self;
        (desc, buffer)
    }
}
