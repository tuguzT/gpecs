use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr, slice,
};

use crate::{
    error::InsufficientAlignError,
    field::{
        ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
        ErasedFieldSlicePtr,
        error::{DowncastError, SlicePtrError},
    },
    ptr::slice::{CastConstPtr, MutSliceItemPtr},
    soa::field::FieldDescriptor,
};

pub struct ErasedFieldSliceMut<'a, T>
where
    T: MutSliceItemPtr,
{
    ptr: ErasedFieldSliceMutPtr<T>,
    phantom: PhantomData<&'a mut [T::Item]>,
}

impl<T> ErasedFieldSliceMut<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_parts(ptr: ErasedFieldMutPtr<T>, len: usize) -> Self {
        let ptr = unsafe { ErasedFieldSliceMutPtr::from_parts(ptr, len) };
        unsafe { Self::from_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldSliceMutPtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
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
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr<CastConstPtr<T>> {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr<CastConstPtr<T>> {
        let Self { ptr, .. } = self;
        ptr.field_ptr().cast_const()
    }

    #[inline]
    pub fn as_mut_field_slice_ptr(&mut self) -> ErasedFieldSliceMutPtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_mut_field_ptr(&mut self) -> ErasedFieldMutPtr<T> {
        let Self { ptr, .. } = self;
        ptr.field_ptr()
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedFieldMutPtr<T>, usize) {
        let Self { ptr, .. } = self;
        ptr.into_parts()
    }
}

impl<'a, T, U> ErasedFieldSliceMut<'a, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(
        desc: FieldDescriptor,
        buffer: &'a mut [U],
        len: usize,
    ) -> Result<Self, SlicePtrError> {
        let ptr = ErasedFieldSliceMutPtr::new(desc, buffer, len)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
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
    pub fn as_uninit_buffer(&self) -> &[MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_uninit_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_uninit_buffer(&mut self) -> &mut [MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_uninit_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn byte_offset(self) -> usize {
        let Self { ptr, .. } = self;
        ptr.byte_offset()
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
    pub fn as_mut_buffer(&mut self) -> &mut [U] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut U {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [U] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }
}

impl<T, U> Debug for ErasedFieldSliceMut<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.as_buffer();
        let len = &self.len();
        f.debug_struct("ErasedFieldSliceMut")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl<T, U> AsRef<[U]> for ErasedFieldSliceMut<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_buffer()
    }
}

impl<T, U> AsMut<[U]> for ErasedFieldSliceMut<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_buffer()
    }
}

impl<'a, T, U, V> TryFrom<&'a mut [V]> for ErasedFieldSliceMut<'a, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(slice: &'a mut [V]) -> Result<Self, Self::Error> {
        let ptr = ptr::from_mut(slice).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }
}

impl<'a, T> From<ErasedFieldSliceMut<'a, T>> for ErasedFieldSlice<'a, CastConstPtr<T>>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn from(value: ErasedFieldSliceMut<'a, T>) -> Self {
        let ptr = value.as_field_slice_ptr();
        unsafe { ErasedFieldSlice::from_ptr(ptr) }
    }
}
