use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Range,
    ptr, slice,
};

use crate::{
    data::{
        ErasedMutPtr, ErasedPtr, ErasedRef,
        error::{DataError, DowncastError, TryFromPtrError},
    },
    ptr::slice::{CastConstPtr, MutSliceItemPtr},
};

pub struct ErasedMutRef<'a, T>
where
    T: MutSliceItemPtr,
{
    ptr: ErasedMutPtr<T>,
    phantom: PhantomData<&'a mut [T::Item]>,
}

impl<T> ErasedMutRef<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedMutPtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { ptr, .. } = self;
        ptr.layout()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedPtr<CastConstPtr<T>> {
        let Self { ptr, .. } = *self;
        ptr.cast_const()
    }
}

impl<'a, T, U> ErasedMutRef<'a, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(layout: Layout, buffer: &'a mut [U]) -> Result<Self, DataError> {
        let ptr = ErasedMutPtr::new(layout, buffer)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub fn try_from<V>(r#ref: &'a mut V) -> Result<Self, TryFromPtrError> {
        let ptr = ptr::from_mut(r#ref).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<&'a mut V, DowncastError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let ptr = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let r#ref = unsafe { ptr.as_mut().unwrap_unchecked() };
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
    pub unsafe fn downcast_mut<V>(&mut self) -> Result<&mut V, DowncastError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let r#ref = unsafe { ptr.as_mut().unwrap_unchecked() };
        Ok(r#ref)
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
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const U {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr().cast_const()
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
    pub fn as_mut_field_ptr(&mut self) -> ErasedMutPtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [U] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }
}

impl<T, U> Debug for ErasedMutRef<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layout = &self.layout();
        let buffer = self.as_buffer();
        f.debug_struct("ErasedFieldRefMut")
            .field("layout", layout)
            .field("buffer", &buffer)
            .finish()
    }
}

impl<T, U> AsRef<[U]> for ErasedMutRef<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_buffer()
    }
}

impl<T, U> AsMut<[U]> for ErasedMutRef<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_buffer()
    }
}

impl<'a, T> From<ErasedMutRef<'a, T>> for ErasedRef<'a, CastConstPtr<T>>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn from(value: ErasedMutRef<'a, T>) -> Self {
        let ptr = value.as_field_ptr();
        unsafe { ErasedRef::from_ptr(ptr) }
    }
}
