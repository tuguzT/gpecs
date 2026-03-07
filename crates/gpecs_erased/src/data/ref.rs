use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    data::{
        ErasedPtr,
        error::{DataError, DowncastError, TryFromPtrError},
    },
    ptr::slice::ConstSliceItemPtr,
};

pub struct ErasedRef<'a, T>
where
    T: ConstSliceItemPtr,
{
    ptr: ErasedPtr<T>,
    phantom: PhantomData<&'a [T::Item]>,
}

impl<'a, T> ErasedRef<'a, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn new(layout: Layout, buffer: &'a [T::Item]) -> Result<Self, DataError> {
        let ptr = ErasedPtr::new(layout, buffer)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub fn try_from<V>(r#ref: &'a V) -> Result<Self, TryFromPtrError> {
        let ptr = ptr::from_ref(r#ref).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedPtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
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
    pub fn layout(&self) -> Layout {
        let Self { ptr, .. } = self;
        ptr.layout()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedPtr<T> {
        let Self { ptr, .. } = *self;
        ptr
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
    pub fn into_buffer(self) -> &'a [T::Item] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }
}

impl<T> Debug for ErasedRef<'_, T>
where
    T: ConstSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layout = &self.layout();
        let buffer = &self.as_buffer();
        f.debug_struct("ErasedFieldRef")
            .field("layout", layout)
            .field("buffer", buffer)
            .finish()
    }
}

impl<T> Clone for ErasedRef<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ErasedRef<'_, T> where T: ConstSliceItemPtr {}

impl<T> AsRef<[T::Item]> for ErasedRef<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_buffer()
    }
}
