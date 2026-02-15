use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Range,
    ptr, slice,
};

use crate::{
    error::InsufficientAlignError,
    field::{
        ErasedFieldPtr,
        error::{DowncastError, PtrError},
    },
    ptr::slice::ConstSliceItemPtr,
    soa::field::FieldDescriptor,
};

pub struct ErasedFieldRef<'a, T>
where
    T: ConstSliceItemPtr,
{
    ptr: ErasedFieldPtr<T>,
    phantom: PhantomData<&'a [T::Item]>,
}

impl<T> ErasedFieldRef<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldPtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }
}

impl<'a, T, U> ErasedFieldRef<'a, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(desc: FieldDescriptor, buffer: &'a [U]) -> Result<Self, PtrError> {
        let ptr = ErasedFieldPtr::new(desc, buffer)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub fn try_from<V>(r#ref: &'a V) -> Result<Self, InsufficientAlignError> {
        let ptr = ptr::from_ref(r#ref).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<&'a V, DowncastError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let ptr = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let r#ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn downcast_ref<V>(&self) -> Result<&V, DowncastError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let r#ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub fn as_uninit_buffer(&self) -> &[MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_uninit_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn byte_offset(&self) -> usize {
        let Self { ptr, .. } = self;
        ptr.byte_offset()
    }

    #[inline]
    pub fn buffer_init_range(&self) -> Range<usize> {
        let Self { ptr, .. } = self;
        ptr.buffer_init_range()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[U] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const U {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [U] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }
}

impl<T, U> Debug for ErasedFieldRef<'_, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
    U: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.as_buffer();
        f.debug_struct("ErasedFieldRef")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<T> Clone for ErasedFieldRef<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ErasedFieldRef<'_, T> where T: ConstSliceItemPtr {}

impl<T, U> AsRef<[U]> for ErasedFieldRef<'_, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_buffer()
    }
}
