use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    mem::ManuallyDrop,
    ptr, slice,
};

use crate::traits::FieldDescriptor;

use super::{
    super::{assert::validate_layout, byte::ErasedByte},
    assert::{assert_layout, assert_value_buffer_len},
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
};

pub struct ErasedField<Fields> {
    desc: FieldDescriptor,
    buffer: Box<[ErasedByte<Fields>]>,
}

impl<Fields> ErasedField<Fields> {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: &[u8]) -> Self {
        validate_layout::<Fields>(desc.layout());
        assert_value_buffer_len(buffer.len(), desc.layout().size());

        let buffer_len = buffer.len().div_ceil(size_of::<ErasedByte<Fields>>());
        let mut r#box = Box::new_uninit_slice(buffer_len);
        unsafe {
            ptr::copy_nonoverlapping(
                buffer.as_ptr(),
                r#box.as_mut_ptr().cast(),
                desc.layout().size(),
            );
        }

        Self {
            desc,
            buffer: unsafe { r#box.assume_init() },
        }
    }

    #[inline]
    pub fn from<T>(value: T) -> Self {
        let value = ManuallyDrop::new(value);
        let desc = FieldDescriptor::of::<T>();
        let data = ptr::from_ref(&value).cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        Self::new(desc, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self) -> T {
        let me = ManuallyDrop::new(self);
        let buffer = unsafe { ptr::read(&me.buffer) };
        let desc = unsafe { ptr::read(&me.desc) };
        assert_layout::<T>(desc.layout());

        unsafe { ptr::read(buffer.as_ptr().cast()) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> &T {
        let Self { desc, buffer } = self;
        assert_layout::<T>(desc.layout());

        let ptr = buffer.as_ptr().cast();
        unsafe { &*ptr }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast_mut<T>(&mut self) -> &mut T {
        let Self { desc, buffer } = self;
        assert_layout::<T>(desc.layout());

        let ptr = buffer.as_mut_ptr().cast();
        unsafe { &mut *ptr }
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { desc, buffer } = self;

        let data = buffer.as_ptr().cast();
        unsafe { slice::from_raw_parts(data, desc.layout().size()) }
    }

    #[inline]
    pub fn as_field_ref(&self) -> ErasedFieldRef<'_> {
        let Self { desc, .. } = *self;
        let buffer = self.buffer();
        ErasedFieldRef::new(desc, buffer)
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.as_ptr().cast()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, .. } = *self;
        let buffer = ptr::from_ref(self.buffer());
        ErasedFieldPtr::new(desc, buffer)
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { desc, buffer } = self;

        let data = buffer.as_mut_ptr().cast();
        unsafe { slice::from_raw_parts_mut(data, desc.layout().size()) }
    }

    #[inline]
    pub fn as_field_ref_mut(&mut self) -> ErasedFieldRefMut<'_> {
        let Self { desc, .. } = *self;
        let buffer = self.buffer_mut();
        ErasedFieldRefMut::new(desc, buffer)
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.as_mut_ptr().cast()
    }

    #[inline]
    pub fn as_field_ptr_mut(&mut self) -> ErasedFieldMutPtr {
        let Self { desc, .. } = *self;
        let buffer = ptr::from_mut(self.buffer_mut());
        ErasedFieldMutPtr::new(desc, buffer)
    }

    #[inline]
    pub fn into_buffer(self) -> Box<[u8]> {
        let (_, buffer) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, Box<[u8]>) {
        let me = ManuallyDrop::new(self);
        let desc = unsafe { ptr::read(&me.desc) };
        let buffer = unsafe { ptr::read(&me.buffer) };

        let data = buffer.as_ptr().cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        (desc, buffer.into())
    }
}

impl<Fields> Debug for ErasedField<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { desc, .. } = self;
        let buffer = self.buffer();
        f.debug_struct("ErasedField")
            .field("desc", desc)
            .field("buffer", &buffer)
            .finish()
    }
}

impl<Fields> Drop for ErasedField<Fields> {
    fn drop(&mut self) {
        // TODO: return drop when the source of double free is found
        let Self { desc, buffer } = self;
        let Some(drop_fn) = desc.drop_fn() else {
            return;
        };

        let data = buffer.as_mut_ptr().cast();
        unsafe { drop_fn(data) }
    }
}
