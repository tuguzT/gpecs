use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    data::{
        ErasedNonNullPtr, ErasedPtr,
        error::{DataError, DowncastError, UpcastPtrError},
    },
    ptr::slice::{ConstPtr, NonNullSliceItemPtr, SliceItemPtr},
};

pub struct ErasedRef<'a, T>
where
    T: SliceItemPtr,
{
    ptr: ErasedNonNullPtr<T>,
    phantom: PhantomData<&'a [T::Item]>,
}

impl<T> ErasedRef<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    pub unsafe fn from_nonnull_ptr(ptr: ErasedNonNullPtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { ptr, .. } = self;
        ptr.layout()
    }

    #[inline]
    pub fn as_nonnull_ptr(&self) -> ErasedNonNullPtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }
}

impl<'a, T> ErasedRef<'a, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedPtr<ConstPtr<T::Ptrs>>) -> Option<Self> {
        let ptr = ptr.cast_nonnull()?;
        let me = unsafe { Self::from_nonnull_ptr(ptr) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_ptr_unchecked(ptr: ErasedPtr<ConstPtr<T::Ptrs>>) -> Self {
        let ptr = unsafe { ptr.cast_nonnull_unchecked() };
        unsafe { Self::from_nonnull_ptr(ptr) }
    }

    #[inline]
    pub fn new(layout: Layout, items: &'a [T::Item]) -> Result<Self, DataError> {
        let items = NonNull::from_ref(items);
        let ptr = ErasedNonNullPtr::new(layout, items)?;
        let me = unsafe { Self::from_nonnull_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub fn upcast<V>(r#ref: &'a V) -> Result<Self, UpcastPtrError> {
        let ptr = NonNull::from_ref(r#ref);
        let ptr = ErasedNonNullPtr::upcast(ptr)?;
        let me = unsafe { Self::from_nonnull_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<&'a V, DowncastError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_nonnull_ptr(ptr) };
        let ptr = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let r#ref = unsafe { ptr.as_ref() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn downcast_ref<V>(&self) -> Result<&V, DowncastError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let r#ref = unsafe { ptr.as_ref() };
        Ok(r#ref)
    }

    #[inline]
    pub fn as_ptr(&self) -> ErasedPtr<ConstPtr<T::Ptrs>> {
        let Self { ptr, .. } = *self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_items(&self) -> &[T::Item] {
        let Self { ptr, .. } = self;

        let items = ptr.as_items();
        unsafe { items.as_ref() }
    }

    #[inline]
    pub fn as_nonnull_item_ptr(&self) -> NonNull<T::Item> {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr()
    }

    #[inline]
    pub fn as_item_ptr(&self) -> *const T::Item {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr().as_ptr()
    }

    #[inline]
    pub fn into_items(self) -> &'a [T::Item] {
        let Self { ptr, .. } = self;

        let items = ptr.as_items();
        unsafe { items.as_ref() }
    }
}

impl<T> Debug for ErasedRef<'_, T>
where
    T: NonNullSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layout = &self.layout();
        let items = &self.as_items();
        f.debug_struct("ErasedRef")
            .field("layout", layout)
            .field("items", items)
            .finish()
    }
}

impl<T> Clone for ErasedRef<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ErasedRef<'_, T> where T: SliceItemPtr {}

impl<T> AsRef<[T::Item]> for ErasedRef<'_, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_items()
    }
}
