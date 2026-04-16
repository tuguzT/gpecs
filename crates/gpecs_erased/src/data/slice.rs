use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr,
};

use crate::{
    data::{
        ErasedPtr, ErasedSlicePtr,
        error::{DataError, DowncastError, TryFromSlicePtrError},
    },
    ptr::slice::ConstSliceItemPtr,
};

pub struct ErasedSlice<'a, T>
where
    T: ConstSliceItemPtr,
{
    ptr: ErasedSlicePtr<T>,
    phantom: PhantomData<&'a [T::Item]>,
}

impl<'a, T> ErasedSlice<'a, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn new(layout: Layout, buffer: &'a [T::Item], len: usize) -> Result<Self, DataError> {
        let ptr = ErasedSlicePtr::new(layout, buffer, len)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(ptr: ErasedPtr<T>, len: usize) -> Self {
        let ptr = unsafe { ErasedSlicePtr::from_parts(ptr, len) };
        unsafe { Self::from_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedSlicePtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<&'a [V], DowncastError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let buffer = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let slice = unsafe { buffer.as_ref_unchecked() };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn downcast_ref<V>(&self) -> Result<&[V], DowncastError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let slice = unsafe { buffer.as_ref_unchecked() };
        Ok(slice)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { ptr, .. } = self;
        ptr.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { ptr, .. } = self;
        ptr.layout()
    }

    #[inline]
    pub fn as_field_slice_ptr(&self) -> ErasedSlicePtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedPtr<T> {
        let Self { ptr, .. } = self;
        ptr.field_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[T::Item] {
        let Self { ptr, .. } = self;

        let buffer = ptr.as_buffer();
        unsafe { buffer.as_ref_unchecked() }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [T::Item] {
        let Self { ptr, .. } = self;

        let buffer = ptr.as_buffer();
        unsafe { buffer.as_ref_unchecked() }
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedPtr<T>, usize) {
        let Self { ptr, .. } = self;
        ptr.into_parts()
    }
}

impl<T> Debug for ErasedSlice<'_, T>
where
    T: ConstSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.layout();
        let buffer = &self.as_buffer();
        let len = &self.len();
        f.debug_struct("ErasedFieldSlice")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl<T> Clone for ErasedSlice<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ErasedSlice<'_, T> where T: ConstSliceItemPtr {}

impl<T> AsRef<[T::Item]> for ErasedSlice<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_buffer()
    }
}

impl<'a, T, V> TryFrom<&'a [V]> for ErasedSlice<'a, T>
where
    T: ConstSliceItemPtr,
{
    type Error = TryFromSlicePtrError;

    #[inline]
    fn try_from(slice: &'a [V]) -> Result<Self, Self::Error> {
        let ptr = ptr::from_ref(slice).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }
}
