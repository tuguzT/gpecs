use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    data::{
        ErasedMutPtr, ErasedNonNullPtr, ErasedPtr, ErasedRef,
        error::{DataError, DowncastError, UpcastPtrError},
    },
    ptr::slice::{ConstPtr, MutPtr, NonNullSliceItemPtr, SliceItemPtr},
};

pub struct ErasedMutRef<'a, T>
where
    T: SliceItemPtr,
{
    ptr: ErasedNonNullPtr<T>,
    phantom: PhantomData<&'a mut [T::Item]>,
}

impl<'a, T> ErasedMutRef<'a, T>
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

    #[inline]
    pub fn as_ref(&self) -> ErasedRef<'_, T> {
        let Self { ptr, .. } = self;
        unsafe { ptr.as_ref() }
    }

    #[inline]
    pub fn into_ref(self) -> ErasedRef<'a, T> {
        let Self { ptr, .. } = self;
        unsafe { ptr.as_ref() }
    }
}

impl<'a, T> ErasedMutRef<'a, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_mut_ptr(ptr: ErasedMutPtr<MutPtr<T::Ptrs>>) -> Option<Self> {
        let ptr = ptr.cast_nonnull()?;
        let me = unsafe { Self::from_nonnull_ptr(ptr) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_mut_ptr_unchecked(ptr: ErasedMutPtr<MutPtr<T::Ptrs>>) -> Self {
        let ptr = unsafe { ptr.cast_nonnull_unchecked() };
        unsafe { Self::from_nonnull_ptr(ptr) }
    }

    #[inline]
    pub fn new(layout: Layout, items: &'a mut [T::Item]) -> Result<Self, DataError> {
        let items = NonNull::from_mut(items);
        let ptr = ErasedNonNullPtr::new(layout, items)?;
        let me = unsafe { Self::from_nonnull_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub fn upcast<V>(r#ref: &'a mut V) -> Result<Self, UpcastPtrError> {
        let ptr = NonNull::from_mut(r#ref);
        let ptr = ErasedNonNullPtr::upcast(ptr)?;
        let me = unsafe { Self::from_nonnull_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<&'a mut V, DowncastError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_nonnull_ptr(ptr) };
        let ptr = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let r#ref = unsafe { ptr.as_mut() };
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
    pub unsafe fn downcast_mut<V>(&mut self) -> Result<&mut V, DowncastError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let r#ref = unsafe { ptr.as_mut() };
        Ok(r#ref)
    }

    #[inline]
    pub fn as_ptr(&self) -> ErasedPtr<ConstPtr<T::Ptrs>> {
        let Self { ptr, .. } = *self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> ErasedMutPtr<MutPtr<T::Ptrs>> {
        let Self { ptr, .. } = *self;
        ptr.cast_mut()
    }

    #[inline]
    pub fn as_items(&self) -> &[T::Item] {
        let Self { ptr, .. } = self;

        let items = ptr.as_items();
        unsafe { items.as_ref() }
    }

    #[inline]
    pub fn as_item_ptr(&self) -> *const T::Item {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr().as_ptr().cast_const()
    }

    #[inline]
    pub fn as_mut_items(&mut self) -> &mut [T::Item] {
        let Self { ptr, .. } = self;

        let mut items = ptr.as_items();
        unsafe { items.as_mut() }
    }

    #[inline]
    pub fn as_mut_item_ptr(&mut self) -> *mut T::Item {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr().as_ptr()
    }

    #[inline]
    pub fn into_items(self) -> &'a [T::Item] {
        let Self { ptr, .. } = self;

        let items = ptr.as_items();
        unsafe { items.as_ref() }
    }

    #[inline]
    pub fn into_mut_items(self) -> &'a mut [T::Item] {
        let Self { ptr, .. } = self;

        let mut items = ptr.as_items();
        unsafe { items.as_mut() }
    }
}

impl<T> Debug for ErasedMutRef<'_, T>
where
    T: NonNullSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layout = &self.layout();
        let items = &self.as_items();
        f.debug_struct("ErasedMutRef")
            .field("layout", layout)
            .field("items", items)
            .finish()
    }
}

impl<T> AsRef<[T::Item]> for ErasedMutRef<'_, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_items()
    }
}

impl<T> AsMut<[T::Item]> for ErasedMutRef<'_, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T::Item] {
        self.as_mut_items()
    }
}
