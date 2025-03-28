use core::{ptr, slice};

use crate::traits::FieldDescriptor;

use super::{
    super::assert::{check_same_layout, check_same_len},
    assert::{check_buffer_align, check_layout},
    error::{ErasedFieldError, IntoValueError},
    ErasedFieldMutPtr, ErasedFieldRef,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldPtr {
    desc: FieldDescriptor,
    ptr: *const u8,
}

impl ErasedFieldPtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: *const [u8]) -> Result<Self, ErasedFieldError> {
        let ptr = buffer.cast();
        check_buffer_align(ptr, desc.layout())?;
        check_same_len(buffer.len(), desc.layout().size())?;

        Ok(Self { desc, ptr })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *const [u8]) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
        }

        let ptr = buffer.cast();
        Self { desc, ptr }
    }

    #[inline]
    pub fn dangling(desc: FieldDescriptor) -> Self {
        let data = ptr::without_provenance(desc.layout().align());
        let buffer = ptr::slice_from_raw_parts(data, desc.layout().size());
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn from<T>(ptr: *const T) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), desc.layout().size());
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into<T>(self) -> Result<*const T, IntoValueError<Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, .. } = me;
        Ok(ptr.cast())
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldMutPtr {
        let Self { desc, ptr } = self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast_mut(), desc.layout().size());
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, ptr } = self;

        let data = unsafe { ptr.add(count * desc.layout().size()) };
        let len = desc.layout().size();
        let buffer = ptr::slice_from_raw_parts(data, len);
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { desc, ptr } = self;
        check_same_layout(origin.descriptor().layout(), desc.layout())
            .expect("layouts should match");

        let offset = unsafe { ptr.offset_from(origin.as_ptr()) };
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
        let Self { desc, ptr } = self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size()) };
        unsafe { ErasedFieldRef::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> *const [u8] {
        let Self { desc, ptr } = *self;
        ptr::slice_from_raw_parts(ptr, desc.layout().size())
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn into_ptr(self) -> *const u8 {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *const [u8]) {
        let Self { desc, ptr } = self;
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
        (desc, buffer)
    }
}
