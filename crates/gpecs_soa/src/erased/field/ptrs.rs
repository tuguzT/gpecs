use core::{ptr, slice};

use crate::traits::FieldDescriptor;

use super::{
    super::assert::assert_layouts,
    assert::{assert_buffer_align, assert_layout, assert_value_buffer_len},
    ErasedFieldMutPtr, ErasedFieldRef,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldPtr {
    desc: FieldDescriptor,
    buffer: *const [u8],
}

impl ErasedFieldPtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: *const [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), desc.layout().size());
        assert_buffer_align(buffer.cast(), desc.layout().align());

        Self { desc, buffer }
    }

    #[inline]
    pub fn dangling(desc: FieldDescriptor) -> Self {
        let data = ptr::without_provenance(desc.layout().align());
        let buffer = ptr::slice_from_raw_parts(data, desc.layout().size());
        Self::new(desc, buffer)
    }

    #[inline]
    pub fn from<T>(ptr: *const T) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), desc.layout().size());
        Self::new(desc, buffer)
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> *const T {
        let Self { desc, buffer } = self;
        assert_layout::<T>(desc.layout());

        buffer.cast()
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldMutPtr {
        let Self { desc, buffer } = self;
        ErasedFieldMutPtr::new(desc, buffer.cast_mut())
    }

    #[inline]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, buffer } = self;

        let data = unsafe { buffer.cast::<u8>().add(count * desc.layout().size()) };
        let len = desc.layout().size();
        let buffer = ptr::slice_from_raw_parts(data, len);
        Self::new(desc, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { desc, .. } = self;
        assert_layouts(desc.layout(), origin.descriptor().layout());

        let offset = unsafe { self.as_ptr().offset_from(origin.as_ptr()) };
        let field_size = desc
            .layout()
            .size()
            .try_into()
            .expect("layout size should not exceed `isize::MAX`");
        offset
            .checked_div(field_size)
            .expect("erased field pointer should not be a ZST")
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldRef<'a> {
        let Self { desc, buffer } = self;
        let buffer = unsafe { slice::from_raw_parts(buffer.cast(), desc.layout().size()) };
        ErasedFieldRef::new(desc, buffer)
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> *const [u8] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_ptr(self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *const [u8]) {
        let Self { desc, buffer } = self;
        (desc, buffer)
    }
}
