use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::traits::FieldDescriptor;

use super::{
    assert::{assert_buffer_align, assert_layout, assert_value_buffer_len},
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef,
};

pub struct ErasedFieldRefMut<'a> {
    desc: FieldDescriptor,
    ptr: *mut u8,
    phantom: PhantomData<&'a mut [u8]>,
}

impl<'a> ErasedFieldRefMut<'a> {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: &'a mut [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), desc.layout().size());
        assert_buffer_align(buffer.as_ptr(), desc.layout().align());

        let ptr = buffer.as_mut_ptr();
        Self {
            desc,
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(r#ref: &'a mut T) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let data = ptr::from_mut(r#ref).cast();
        let buffer = unsafe { slice::from_raw_parts_mut(data, desc.layout().size()) };
        Self::new(desc, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self) -> &'a mut T {
        let Self { desc, ptr, .. } = self;
        assert_layout::<T>(desc.layout());

        let ptr = ptr.cast();
        unsafe { &mut *ptr }
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
    #[track_caller]
    pub unsafe fn cast_mut<T>(&mut self) -> &mut T {
        let Self { desc, ptr, .. } = self;
        assert_layout::<T>(desc.layout());

        let ptr = ptr.cast();
        unsafe { &mut *ptr }
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
        ErasedFieldPtr::new(desc, buffer)
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { desc, ptr, .. } = *self;
        unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_mut_ptr(&mut self) -> ErasedFieldMutPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size()) };
        ErasedFieldMutPtr::new(desc, buffer)
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [u8] {
        let (_, buffer) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a mut [u8]) {
        let Self { desc, ptr, .. } = self;
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size()) };
        (desc, buffer)
    }
}

impl Debug for ErasedFieldRefMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { desc, .. } = self;
        let buffer = &self.buffer();
        f.debug_struct("ErasedFieldRefMut")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<'a> From<ErasedFieldRefMut<'a>> for ErasedFieldRef<'a> {
    fn from(value: ErasedFieldRefMut<'a>) -> Self {
        let (desc, buffer) = value.into_parts();
        ErasedFieldRef::new(desc, buffer)
    }
}
