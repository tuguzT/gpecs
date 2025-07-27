use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    error::{check_align, check_len},
    soa::traits::FieldDescriptor,
};

use super::{
    ErasedFieldPtr,
    assert::check_into_layout,
    error::{ErasedFieldPtrError, IntoValueError},
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
    pub fn new(desc: FieldDescriptor, buffer: &'a [u8]) -> Result<Self, ErasedFieldPtrError> {
        let ptr = buffer.as_ptr();
        check_len(buffer.len(), desc.layout().size())?;
        check_align(ptr, desc.layout())?;

        Ok(Self {
            desc,
            ptr,
            phantom: PhantomData,
        })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a [u8]) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
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
    pub unsafe fn into<T>(self) -> Result<&'a T, IntoValueError<Self>> {
        let me = check_into_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, .. } = me;

        let ptr = ptr.cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&T, IntoValueError<&Self>> {
        let me = check_into_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, .. } = me;

        let ptr = ptr.cast();
        Ok(unsafe { &*ptr })
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

impl AsRef<[u8]> for ErasedFieldRef<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer()
    }
}
