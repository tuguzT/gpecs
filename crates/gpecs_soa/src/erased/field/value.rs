use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    mem::ManuallyDrop,
    ptr, slice,
};

use crate::traits::FieldDescriptor;

use super::{
    super::{
        assert::validate_layout,
        byte::{Aligned, ErasedByte, Fields, Unaligned},
    },
    assert::{assert_layout, assert_value_buffer_len},
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
};

pub struct ErasedField<F>
where
    F: Fields,
{
    desc: FieldDescriptor,
    buffer: Box<[ErasedByte<F>]>,
}

impl<F> ErasedField<F>
where
    F: Fields,
{
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: &[u8]) -> Self {
        if F::ALIGNED {
            validate_layout::<F>(desc.layout());
        }
        assert_value_buffer_len(buffer.len(), desc.layout().size());

        let buffer_len = buffer.len().div_ceil(size_of::<ErasedByte<F>>());
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
        let desc = unsafe { ptr::read(&me.desc) };
        let buffer = unsafe { ptr::read(&me.buffer) };
        assert_layout::<T>(desc.layout());

        let src = buffer.as_ptr().cast();
        if F::ALIGNED {
            unsafe { ptr::read(src) }
        } else {
            unsafe { ptr::read_unaligned(src) }
        }
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { ref desc, buffer } = self;

        let data = buffer.as_ptr().cast();
        unsafe { slice::from_raw_parts(data, desc.layout().size()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.as_ptr().cast()
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { ref desc, buffer } = self;

        let data = buffer.as_mut_ptr().cast();
        unsafe { slice::from_raw_parts_mut(data, desc.layout().size()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.as_mut_ptr().cast()
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

impl<F> ErasedField<Aligned<F>> {
    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> &T {
        let Self { ref desc, buffer } = self;
        assert_layout::<T>(desc.layout());

        let ptr = buffer.as_ptr().cast();
        unsafe { &*ptr }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast_mut<T>(&mut self) -> &mut T {
        let Self { ref desc, buffer } = self;
        assert_layout::<T>(desc.layout());

        let ptr = buffer.as_mut_ptr().cast();
        unsafe { &mut *ptr }
    }

    #[inline]
    pub fn as_field_ref(&self) -> ErasedFieldRef<'_> {
        let Self { desc, .. } = *self;
        let buffer = self.buffer();
        unsafe { ErasedFieldRef::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, .. } = *self;
        let buffer = ptr::from_ref(self.buffer());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ref_mut(&mut self) -> ErasedFieldRefMut<'_> {
        let Self { desc, .. } = *self;
        let buffer = self.buffer_mut();
        unsafe { ErasedFieldRefMut::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ptr_mut(&mut self) -> ErasedFieldMutPtr {
        let Self { desc, .. } = *self;
        let buffer = ptr::from_mut(self.buffer_mut());
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_unaligned(self) -> ErasedField<Unaligned> {
        let (desc, buffer) = self.into_parts();
        ErasedField::new(desc, &buffer)
    }
}

impl ErasedField<Unaligned> {
    #[inline]
    pub fn into_aligned<F>(self) -> ErasedField<Aligned<F>> {
        let (desc, buffer) = self.into_parts();
        ErasedField::new(desc, &buffer)
    }
}

impl<F> Debug for ErasedField<F>
where
    F: Fields,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { desc, .. } = self;
        let buffer = &self.buffer();
        let aligned = &F::ALIGNED;
        f.debug_struct("ErasedField")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("aligned", aligned)
            .finish()
    }
}
