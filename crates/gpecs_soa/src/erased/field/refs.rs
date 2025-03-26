use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::traits::FieldDescriptor;

use super::{
    assert::{assert_buffer_align, assert_layout, assert_value_buffer_len},
    ErasedFieldPtr,
};

#[derive(Clone, Copy)]
pub struct ErasedFieldRef<'a> {
    desc: FieldDescriptor,
    ptr: *const u8,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> ErasedFieldRef<'a> {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: &'a [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), desc.layout().size());
        assert_buffer_align(buffer.as_ptr(), desc.layout().align());

        let ptr = buffer.as_ptr();
        Self {
            desc,
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a [u8]) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer);
        }

        let ptr = buffer.as_ptr();
        Self {
            desc,
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(r#ref: &'a T) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let data = ptr::from_ref(r#ref).cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self) -> &'a T {
        let Self { desc, ptr, .. } = self;
        assert_layout::<T>(desc.layout());

        let ptr = ptr.cast();
        unsafe { &*ptr }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> &T {
        let Self { desc, ptr, .. } = self;
        assert_layout::<T>(desc.layout());

        let ptr = ptr.cast();
        unsafe { &*ptr }
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { desc, ptr, .. } = *self;
        unsafe { slice::from_raw_parts(ptr, desc.layout().size()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size()) };
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [u8] {
        let (_, buffer) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a [u8]) {
        let Self { desc, ptr, .. } = self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size()) };
        (desc, buffer)
    }
}

impl Debug for ErasedFieldRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { desc, .. } = self;
        let buffer = &self.buffer();
        f.debug_struct("ErasedFieldRef")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}
