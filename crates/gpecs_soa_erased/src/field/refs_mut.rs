use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::soa::traits::FieldDescriptor;

use super::{
    super::assert::check_same_len,
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef,
    assert::{check_buffer_align, check_layout},
    error::{ErasedFieldError, IntoValueError},
};

pub struct ErasedFieldRefMut<'a> {
    desc: FieldDescriptor,
    ptr: *mut u8,
    phantom: PhantomData<&'a mut [u8]>,
}

impl<'a> ErasedFieldRefMut<'a> {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: &'a mut [u8]) -> Result<Self, ErasedFieldError> {
        let ptr = buffer.as_mut_ptr();
        check_buffer_align(ptr, desc.layout())?;
        check_same_len(buffer.len(), desc.layout().size())?;

        Ok(Self {
            desc,
            ptr,
            phantom: PhantomData,
        })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a mut [u8]) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
        }

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
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub unsafe fn into<T>(self) -> Result<&'a mut T, IntoValueError<Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, .. } = me;

        let ptr = ptr.cast();
        Ok(unsafe { &mut *ptr })
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&T, IntoValueError<&Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, .. } = me;

        let ptr = ptr.cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut T, IntoValueError<&mut Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, .. } = me;

        let ptr = ptr.cast();
        Ok(unsafe { &mut *ptr })
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
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
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

impl AsRef<[u8]> for ErasedFieldRefMut<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer()
    }
}

impl AsMut<[u8]> for ErasedFieldRefMut<'_> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.buffer_mut()
    }
}

impl<'a> From<ErasedFieldRefMut<'a>> for ErasedFieldRef<'a> {
    fn from(value: ErasedFieldRefMut<'a>) -> Self {
        let (desc, buffer) = value.into_parts();
        unsafe { ErasedFieldRef::new_unchecked(desc, buffer) }
    }
}
