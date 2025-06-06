use core::ptr::{self, NonNull};

use crate::{assert::check_same_layout, soa::traits::FieldDescriptor};

use super::{
    super::assert::check_same_len,
    assert::{check_buffer_align, check_layout},
    error::{ErasedFieldError, IntoValueError},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldNonNullPtr {
    desc: FieldDescriptor,
    ptr: NonNull<u8>,
}

impl ErasedFieldNonNullPtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: NonNull<[u8]>) -> Result<Self, ErasedFieldError> {
        let ptr = buffer.cast();
        check_buffer_align(ptr.as_ptr(), desc.layout())?;
        check_same_len(buffer.len(), desc.layout().size())?;

        Ok(Self { desc, ptr })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: NonNull<[u8]>) -> Self {
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
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn from<T>(ptr: NonNull<T>) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast(), desc.layout().size());
        let buffer = unsafe { NonNull::new_unchecked(ptr) };
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into<T>(self) -> Result<NonNull<T>, IntoValueError<Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, .. } = me;
        Ok(ptr.cast())
    }

    #[inline]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, ptr } = self;

        let data = unsafe { ptr.add(count * desc.layout().size()) };
        let len = desc.layout().size();
        let buffer = ptr::slice_from_raw_parts_mut(data.as_ptr(), len);
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
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
    #[track_caller]
    pub unsafe fn swap(self, with: Self, temp: &mut [u8]) {
        let Self { desc, .. } = self;
        check_same_layout(with.descriptor().layout(), desc.layout()).expect("layouts should match");

        let count = desc.layout().size();
        assert!(temp.len() >= count);

        let a = self.as_ptr();
        let b = with.as_ptr();
        unsafe {
            ptr::copy_nonoverlapping(a.as_ptr(), temp.as_mut_ptr(), count);
            ptr::copy(b.as_ptr(), a.as_ptr(), count);
            ptr::copy_nonoverlapping(temp.as_ptr(), b.as_ptr(), count);
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from(self, from: Self, count: usize, temp: &mut [u8]) {
        let Self { desc, .. } = self;
        check_same_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let count = count * desc.layout().size();
        assert!(temp.len() >= count);

        let src = from.as_ptr();
        let dst = self.as_ptr();
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), temp.as_mut_ptr(), count);
            ptr::copy_nonoverlapping(temp.as_ptr(), dst.as_ptr(), count);
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(self, from: Self, count: usize) {
        let Self { desc, .. } = self;
        check_same_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let count = count * desc.layout().size();
        let src = from.as_ptr();
        let dst = self.as_ptr();
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), count);
        }
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
