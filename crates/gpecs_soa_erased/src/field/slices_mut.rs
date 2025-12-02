use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::{self},
    slice,
};

use crate::{error::check_align, soa::field::FieldDescriptor};

use super::{
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
    ErasedFieldSlicePtr,
    assert::{check_into_layout, check_slice_buffer_len},
    error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
};

pub struct ErasedFieldSliceMut<'a> {
    desc: FieldDescriptor,
    ptr: *mut u8,
    len: usize,
    phantom: PhantomData<&'a mut [u8]>,
}

impl<'a> ErasedFieldSliceMut<'a> {
    #[inline]
    #[track_caller]
    pub fn new(
        desc: FieldDescriptor,
        buffer: &'a mut [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = buffer.as_mut_ptr();
        check_align(ptr, desc.layout())?;
        check_slice_buffer_len(buffer.len(), desc.layout().size(), len)?;

        Ok(Self {
            desc,
            ptr,
            len,
            phantom: PhantomData,
        })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a mut [u8], len: usize) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer, len).expect("incorrect inputs");
        }

        let ptr = buffer.as_mut_ptr();
        Self {
            desc,
            ptr,
            len,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(ptr: &'a mut [T]) -> Self {
        let len = ptr.len();
        let desc = FieldDescriptor::of::<T>();
        let data = ptr.as_mut_ptr().cast();
        let buffer = unsafe { slice::from_raw_parts_mut(data, desc.layout().size() * ptr.len()) };
        unsafe { Self::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub unsafe fn into<T>(self) -> Result<&'a mut [T], ErasedFieldIntoValueError<Self>> {
        let Self { desc, .. } = self;
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { ptr, len, .. } = me;

        let data = ptr.cast();
        Ok(unsafe { slice::from_raw_parts_mut(data, len) })
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&[T], ErasedFieldIntoValueError<&Self>> {
        let Self { desc, .. } = self;
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { ptr, len, .. } = *me;

        let data = ptr.cast();
        Ok(unsafe { slice::from_raw_parts(data, len) })
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut [T], ErasedFieldIntoValueError<&mut Self>> {
        let Self { desc, .. } = self;
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { ptr, len, .. } = *me;

        let data = ptr.cast();
        Ok(unsafe { slice::from_raw_parts_mut(data, len) })
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { desc, ptr, len, .. } = self;
        unsafe { slice::from_raw_parts(*ptr, desc.layout().size() * len) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr {
        let Self { desc, ptr, len, .. } = *self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { desc, ptr, len, .. } = *self;
        unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size() * len) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_slice_mut_ptr(&mut self) -> ErasedFieldSliceMutPtr {
        let Self { desc, ptr, len, .. } = *self;
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub fn as_field_mut_ptr(&mut self) -> ErasedFieldMutPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size());
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [u8] {
        let (_, buffer, _) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a mut [u8], usize) {
        let Self { desc, ptr, len, .. } = self;
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size() * len) };
        (desc, buffer, len)
    }
}

impl Debug for ErasedFieldSliceMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { desc, len, .. } = self;
        let buffer = &self.buffer();
        f.debug_struct("ErasedFieldSliceMut")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl AsRef<[u8]> for ErasedFieldSliceMut<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer()
    }
}

impl AsMut<[u8]> for ErasedFieldSliceMut<'_> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.buffer_mut()
    }
}

impl<'a> From<ErasedFieldSliceMut<'a>> for ErasedFieldSlice<'a> {
    fn from(value: ErasedFieldSliceMut<'a>) -> Self {
        let (desc, buffer, len) = value.into_parts();
        unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
    }
}
