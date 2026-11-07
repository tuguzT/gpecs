use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    data::{
        ErasedMutPtr, ErasedMutSlicePtr, ErasedNonNullPtr, ErasedNonNullSlicePtr, ErasedPtr,
        ErasedSlice, ErasedSlicePtr,
        error::{DataError, DowncastError, UpcastSliceError},
    },
    ptr::slice::{ConstPtr, MutPtr, NonNullSliceItemPtr, SliceItemPtr},
};

pub struct ErasedMutSlice<'a, T>
where
    T: SliceItemPtr,
{
    ptr: ErasedNonNullSlicePtr<T>,
    phantom: PhantomData<&'a mut [T::Item]>,
}

impl<'a, T> ErasedMutSlice<'a, T>
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

    #[inline]
    pub fn as_slice(&self) -> ErasedSlice<'_, T> {
        let Self { ptr, .. } = self;
        unsafe { ptr.as_ref() }
    }

    #[inline]
    pub fn into_slice(self) -> ErasedSlice<'a, T> {
        let Self { ptr, .. } = self;
        unsafe { ptr.as_ref() }
    }
}

impl<'a, T> ErasedMutSlice<'a, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_mut_slice_ptr(ptr: ErasedMutSlicePtr<MutPtr<T::Ptrs>>) -> Option<Self> {
        let ptr = ptr.cast_nonnull()?;
        let me = unsafe { Self::from_nonnull_slice_ptr(ptr) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_mut_slice_ptr_unchecked(ptr: ErasedMutSlicePtr<MutPtr<T::Ptrs>>) -> Self {
        let ptr = unsafe { ptr.cast_nonnull_unchecked() };
        unsafe { Self::from_nonnull_slice_ptr(ptr) }
    }

    #[inline]
    pub fn new(layout: Layout, items: &'a mut [T::Item], len: usize) -> Result<Self, DataError> {
        let items = NonNull::from_mut(items);
        let ptr = ErasedNonNullSlicePtr::new(layout, items, len)?;
        let me = unsafe { Self::from_nonnull_slice_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub fn upcast<V>(slice: &'a mut [V]) -> Result<Self, UpcastSliceError> {
        let ptr = NonNull::from_mut(slice);
        let ptr = ErasedNonNullSlicePtr::upcast(ptr)?;
        let me = unsafe { Self::from_nonnull_slice_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<V>(self) -> Result<&'a mut [V], DowncastError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_nonnull_slice_ptr(ptr) };
        let mut slice = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let slice = unsafe { slice.as_mut() };
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
    pub unsafe fn downcast_mut<V>(&mut self) -> Result<&mut [V], DowncastError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let mut slice = ptr
            .downcast::<V>()
            .map_err(|err| err.map_value(into_self))?;

        let slice = unsafe { slice.as_mut() };
        Ok(slice)
    }

    #[inline]
    pub fn as_slice_ptr(&self) -> ErasedSlicePtr<ConstPtr<T::Ptrs>> {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_ptr(&self) -> ErasedPtr<ConstPtr<T::Ptrs>> {
        let Self { ptr, .. } = self;
        ptr.as_ptr().cast_const()
    }

    #[inline]
    pub fn as_mut_slice_ptr(&mut self) -> ErasedMutSlicePtr<MutPtr<T::Ptrs>> {
        let Self { ptr, .. } = *self;
        ptr.cast_mut()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> ErasedMutPtr<MutPtr<T::Ptrs>> {
        let Self { ptr, .. } = self;
        ptr.as_ptr().cast_mut()
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

impl<T> Debug for ErasedMutSlice<'_, T>
where
    T: NonNullSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layout = &self.layout();
        let items = &self.as_items();
        let len = &self.len();
        f.debug_struct("ErasedMutSlice")
            .field("layout", layout)
            .field("items", items)
            .field("len", len)
            .finish()
    }
}

impl<T> AsRef<[T::Item]> for ErasedMutSlice<'_, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_items()
    }
}

impl<T> AsMut<[T::Item]> for ErasedMutSlice<'_, T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T::Item] {
        self.as_mut_items()
    }
}
