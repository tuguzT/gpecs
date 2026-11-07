use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    data::{
        ErasedNonNullPtr, ErasedNonNullSlicePtr, ErasedPtr, ErasedSlicePtr,
        error::{DataError, DowncastError, UpcastSliceError},
    },
    ptr::slice::{ConstPtr, NonNullSliceItemPtr, SliceItemPtr},
};

pub struct ErasedSlice<'a, T>
where
    T: SliceItemPtr,
{
    ptr: ErasedNonNullSlicePtr<T>,
    phantom: PhantomData<&'a [T::Item]>,
}

impl<T> ErasedSlice<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    pub unsafe fn from_parts(ptr: ErasedNonNullPtr<T>, len: usize) -> Self {
        let ptr = unsafe { ErasedNonNullSlicePtr::from_parts(ptr, len) };
        unsafe { Self::from_nonnull_slice_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_nonnull_slice_ptr(ptr: ErasedNonNullSlicePtr<T>) -> Self {
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
    pub fn layout(&self) -> Layout {
        let Self { ptr, .. } = self;
        ptr.layout()
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedNonNullPtr<T>, usize) {
        let Self { ptr, .. } = self;
        ptr.into_parts()
    }

    #[inline]
    pub fn as_nonnull_slice_ptr(&self) -> ErasedNonNullSlicePtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_nonnull_ptr(&self) -> ErasedNonNullPtr<T> {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }
}

impl<'a, T> ErasedSlice<'a, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_slice_ptr(ptr: ErasedSlicePtr<ConstPtr<T::Ptrs>>) -> Option<Self> {
        let ptr = ptr.cast_nonnull()?;
        let me = unsafe { Self::from_nonnull_slice_ptr(ptr) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_slice_ptr_unchecked(ptr: ErasedSlicePtr<ConstPtr<T::Ptrs>>) -> Self {
        let ptr = unsafe { ptr.cast_nonnull_unchecked() };
        unsafe { Self::from_nonnull_slice_ptr(ptr) }
    }

    #[inline]
    pub fn new(layout: Layout, items: &'a [T::Item], len: usize) -> Result<Self, DataError> {
        let items = NonNull::from_ref(items);
        let ptr = ErasedNonNullSlicePtr::new(layout, items, len)?;
        let me = unsafe { Self::from_nonnull_slice_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub fn upcast<V>(slice: &'a [V]) -> Result<Self, UpcastSliceError> {
        let ptr = NonNull::from_ref(slice);
        let ptr = ErasedNonNullSlicePtr::upcast(ptr)?;
        let me = unsafe { Self::from_nonnull_slice_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<&'a [V], DowncastError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_nonnull_slice_ptr(ptr) };
        let slice = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let slice = unsafe { slice.as_ref() };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn downcast_ref<V>(&self) -> Result<&[V], DowncastError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let slice = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let slice = unsafe { slice.as_ref() };
        Ok(slice)
    }

    #[inline]
    pub fn as_slice_ptr(&self) -> ErasedSlicePtr<ConstPtr<T::Ptrs>> {
        let Self { ptr, .. } = *self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_ptr(&self) -> ErasedPtr<ConstPtr<T::Ptrs>> {
        let Self { ptr, .. } = self;
        ptr.as_ptr().cast_const()
    }

    #[inline]
    pub fn as_nonnull_item_ptr(&self) -> NonNull<T::Item> {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr()
    }

    #[inline]
    pub fn as_item_ptr(&self) -> *const T::Item {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr().as_ptr().cast_const()
    }

    #[inline]
    pub fn as_items(&self) -> &[T::Item] {
        let Self { ptr, .. } = self;

        let items = ptr.as_items();
        unsafe { items.as_ref() }
    }

    #[inline]
    pub fn into_items(self) -> &'a [T::Item] {
        let Self { ptr, .. } = self;

        let items = ptr.as_items();
        unsafe { items.as_ref() }
    }
}

impl<T> Debug for ErasedSlice<'_, T>
where
    T: NonNullSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layout = &self.layout();
        let items = &self.as_items();
        let len = &self.len();
        f.debug_struct("ErasedSlice")
            .field("layout", layout)
            .field("items", items)
            .field("len", len)
            .finish()
    }
}

impl<T> Clone for ErasedSlice<'_, T>
where
    T: SliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ErasedSlice<'_, T> where T: SliceItemPtr {}

impl<T> AsRef<[T::Item]> for ErasedSlice<'_, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_items()
    }
}
