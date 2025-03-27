use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    mem::ManuallyDrop,
    ptr, slice,
};

use crate::traits::FieldDescriptor;

use super::{
    super::{
        assert::{check_same_len, validate_layout},
        byte::{Aligned, ErasedByte, Fields, Unaligned},
        error::LenMismatchError,
    },
    assert::check_layout,
    error::LayoutMismatchError,
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
    pub fn new(desc: FieldDescriptor, buffer: &[u8]) -> Result<Self, LenMismatchError> {
        if F::ALIGNED {
            validate_layout::<F>(desc.layout());
        }
        check_same_len(buffer.len(), desc.layout().size())?;

        let me = unsafe { Self::actual_new(desc, buffer) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &[u8]) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(desc, buffer) }
    }

    #[inline]
    unsafe fn actual_new(desc: FieldDescriptor, buffer: &[u8]) -> Self {
        let buffer_len = buffer.len().div_ceil(size_of::<ErasedByte<F>>());
        let mut r#box = Box::new_uninit_slice(buffer_len);
        unsafe {
            ptr::copy_nonoverlapping(
                buffer.as_ptr(),
                r#box.as_mut_ptr().cast(),
                desc.layout().size(),
            );
        }

        let buffer = unsafe { r#box.assume_init() };
        Self { desc, buffer }
    }

    #[inline]
    pub fn from<T>(value: T) -> Self {
        let value = ManuallyDrop::new(value);
        let desc = FieldDescriptor::of::<T>();
        let data = ptr::from_ref(&value).cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub unsafe fn into<T>(self) -> Result<T, LayoutMismatchError<Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { buffer, .. } = me;

        let src = buffer.as_ptr().cast();
        if F::ALIGNED {
            Ok(unsafe { ptr::read(src) })
        } else {
            Ok(unsafe { ptr::read_unaligned(src) })
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
        let Self { desc, buffer } = self;

        let data = buffer.as_ptr().cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        (desc, buffer.into())
    }
}

impl<F> ErasedField<Aligned<F>> {
    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> Result<&T, LayoutMismatchError<&Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { buffer, .. } = me;

        let ptr = buffer.as_ptr().cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut T, LayoutMismatchError<&mut Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { buffer, .. } = me;

        let ptr = buffer.as_mut_ptr().cast();
        Ok(unsafe { &mut *ptr })
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
        unsafe { ErasedField::new_unchecked(desc, &buffer) }
    }
}

impl ErasedField<Unaligned> {
    #[inline]
    pub fn into_aligned<F>(self) -> ErasedField<Aligned<F>> {
        let (desc, buffer) = self.into_parts();
        unsafe { ErasedField::new_unchecked(desc, &buffer) }
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
