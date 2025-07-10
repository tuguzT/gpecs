use core::{ptr, slice};

use crate::soa::traits::FieldDescriptor;

use super::{
    super::assert::{check_same_layout, check_same_len},
    ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
    assert::{check_buffer_align, check_layout},
    error::{ErasedFieldError, IntoValueError},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldMutPtr {
    desc: FieldDescriptor,
    ptr: *mut u8,
}

impl ErasedFieldMutPtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: *mut [u8]) -> Result<Self, ErasedFieldError> {
        let ptr = buffer.cast();
        check_buffer_align(ptr, desc.layout())?;
        check_same_len(buffer.len(), desc.layout().size())?;

        Ok(Self { desc, ptr })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *mut [u8]) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
        }

        let ptr = buffer.cast();
        Self { desc, ptr }
    }

    #[inline]
    pub fn dangling(desc: FieldDescriptor) -> Self {
        let data = ptr::without_provenance_mut(desc.layout().align());
        let buffer = ptr::slice_from_raw_parts_mut(data, desc.layout().size());
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn from<T>(ptr: *mut T) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), desc.layout().size());
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into<T>(self) -> Result<*mut T, IntoValueError<Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, .. } = me;
        Ok(ptr.cast())
    }

    #[inline]
    pub fn cast_const(self) -> ErasedFieldPtr {
        let Self { desc, ptr } = self;
        let buffer = ptr::slice_from_raw_parts(ptr.cast_const(), desc.layout().size());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, ptr } = self;

        let data = unsafe { ptr.add(count * desc.layout().size()) };
        let len = desc.layout().size();
        let buffer = ptr::slice_from_raw_parts_mut(data, len);
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: ErasedFieldPtr) -> isize {
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
    #[track_caller]
    pub unsafe fn swap(self, with: Self, temp: &mut [u8]) {
        let Self { desc, .. } = self;
        check_same_layout(with.descriptor().layout(), desc.layout()).expect("layouts should match");

        let count = desc.layout().size();
        assert!(temp.len() >= count);

        let a = self.as_ptr();
        let b = with.as_ptr();
        unsafe {
            ptr::copy_nonoverlapping(a, temp.as_mut_ptr(), count);
            ptr::copy(b, a, count);
            ptr::copy_nonoverlapping(temp.as_ptr(), b, count);
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from(self, from: ErasedFieldPtr, count: usize, temp: &mut [u8]) {
        let Self { desc, .. } = self;
        check_same_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let count = count * desc.layout().size();
        assert!(temp.len() >= count);

        let src = from.as_ptr();
        let dst = self.as_ptr();
        unsafe {
            ptr::copy_nonoverlapping(src, temp.as_mut_ptr(), count);
            ptr::copy_nonoverlapping(temp.as_ptr(), dst, count);
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(self, from: ErasedFieldPtr, count: usize) {
        let Self { desc, .. } = self;
        check_same_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let count = count * desc.layout().size();
        let src = from.as_ptr();
        let dst = self.as_ptr();
        unsafe {
            ptr::copy_nonoverlapping(src, dst, count);
        }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldRef<'a> {
        let Self { desc, ptr } = self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size()) };
        unsafe { ErasedFieldRef::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedFieldRefMut<'a> {
        let Self { desc, ptr } = self;
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size()) };
        unsafe { ErasedFieldRefMut::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> *mut [u8] {
        let Self { desc, ptr } = *self;
        ptr::slice_from_raw_parts_mut(ptr, desc.layout().size())
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn into_ptr(self) -> *mut u8 {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *mut [u8]) {
        let Self { desc, ptr } = self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size());
        (desc, buffer)
    }
}
