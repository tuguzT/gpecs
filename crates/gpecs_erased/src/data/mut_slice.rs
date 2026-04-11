use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    data::{
        ErasedMutPtr, ErasedMutSlicePtr, ErasedPtr, ErasedSlice, ErasedSlicePtr,
        error::{DataError, DowncastError, TryFromSlicePtrError},
    },
    ptr::slice::{CastConst, MutSliceItemPtr},
};

pub struct ErasedMutSlice<'a, T>
where
    T: MutSliceItemPtr,
{
    ptr: ErasedMutSlicePtr<T>,
    phantom: PhantomData<&'a mut [T::Item]>,
}

impl<'a, T> ErasedMutSlice<'a, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    pub fn new(layout: Layout, buffer: &'a mut [T::Item], len: usize) -> Result<Self, DataError> {
        let ptr = ErasedMutSlicePtr::new(layout, buffer, len)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(ptr: ErasedMutPtr<T>, len: usize) -> Self {
        let ptr = unsafe { ErasedMutSlicePtr::from_parts(ptr, len) };
        unsafe { Self::from_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedMutSlicePtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<&'a mut [V], DowncastError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let buffer = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let slice = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn downcast_ref<V>(&self) -> Result<&[V], DowncastError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn downcast_mut<V>(&mut self) -> Result<&mut [V], DowncastError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let slice = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
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
    pub fn as_field_slice_ptr(&self) -> ErasedSlicePtr<CastConst<T>> {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedPtr<CastConst<T>> {
        let Self { ptr, .. } = self;
        ptr.field_ptr().cast_const()
    }

    #[inline]
    pub fn as_mut_field_slice_ptr(&mut self) -> ErasedMutSlicePtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_mut_field_ptr(&mut self) -> ErasedMutPtr<T> {
        let Self { ptr, .. } = self;
        ptr.field_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[T::Item] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [T::Item] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T::Item {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [T::Item] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedMutPtr<T>, usize) {
        let Self { ptr, .. } = self;
        ptr.into_parts()
    }
}

impl<T> Debug for ErasedMutSlice<'_, T>
where
    T: MutSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.layout();
        let buffer = &self.as_buffer();
        let len = &self.len();
        f.debug_struct("ErasedFieldSliceMut")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl<T> AsRef<[T::Item]> for ErasedMutSlice<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_buffer()
    }
}

impl<T> AsMut<[T::Item]> for ErasedMutSlice<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T::Item] {
        self.as_mut_buffer()
    }
}

impl<'a, T, V> TryFrom<&'a mut [V]> for ErasedMutSlice<'a, T>
where
    T: MutSliceItemPtr,
{
    type Error = TryFromSlicePtrError;

    #[inline]
    fn try_from(slice: &'a mut [V]) -> Result<Self, Self::Error> {
        let ptr = ptr::from_mut(slice).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }
}

impl<'a, T> From<ErasedMutSlice<'a, T>> for ErasedSlice<'a, CastConst<T>>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn from(value: ErasedMutSlice<'a, T>) -> Self {
        let ptr = value.as_field_slice_ptr();
        unsafe { ErasedSlice::from_ptr(ptr) }
    }
}
