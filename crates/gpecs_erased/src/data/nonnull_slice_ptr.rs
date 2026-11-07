use core::{alloc::Layout, ptr::NonNull};

use crate::{
    data::{
        ErasedMutSlice, ErasedMutSlicePtr, ErasedNonNullPtr, ErasedSlice, ErasedSlicePtr,
        error::{DataError, DowncastError, UpcastSliceError, check_downcast},
    },
    error::{InsufficientAlignError, check_len, check_ptr_align, check_sufficient_align},
    layout::bytes_to_items,
    ptr::slice::{ConstPtr, MutPtr, NonNullSliceItemPtr, SliceItemPtr},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedNonNullSlicePtr<T> {
    len: usize,
    ptr: ErasedNonNullPtr<T>,
}

impl<T> ErasedNonNullSlicePtr<T> {
    #[inline]
    pub unsafe fn from_parts(ptr: ErasedNonNullPtr<T>, len: usize) -> Self {
        Self { len, ptr }
    }

    #[inline]
    pub fn len(self) -> usize {
        let Self { len, .. } = self;
        len
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn layout(self) -> Layout {
        let Self { ptr, .. } = self;
        ptr.layout()
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedNonNullPtr<T>, usize) {
        let Self { ptr, len } = self;
        (ptr, len)
    }

    #[inline]
    pub fn as_ptr(self) -> ErasedNonNullPtr<T> {
        let Self { ptr, .. } = self;
        ptr
    }
}

impl<T> ErasedNonNullSlicePtr<T>
where
    T: SliceItemPtr,
{
    #[inline]
    pub unsafe fn as_ref<'a>(self) -> ErasedSlice<'a, T> {
        unsafe { ErasedSlice::from_nonnull_slice_ptr(self) }
    }

    #[inline]
    pub unsafe fn as_mut<'a>(self) -> ErasedMutSlice<'a, T> {
        unsafe { ErasedMutSlice::from_nonnull_slice_ptr(self) }
    }
}

impl<T> ErasedNonNullSlicePtr<T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub fn new(layout: Layout, items: NonNull<[T::Item]>, len: usize) -> Result<Self, DataError> {
        check_ptr_align(items.as_ptr().cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let layout_of_items = Layout::array::<T::Item>(items.len())?;
        let expected_layout = layout.pad_to_align().repeat_packed(len)?;
        check_len(layout_of_items.size(), expected_layout.size())?;

        let ptr = unsafe { T::from_parts(items, 0) };
        let ptr = unsafe { ErasedNonNullPtr::from_parts(layout, ptr) };

        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(layout: Layout) -> Result<Self, InsufficientAlignError> {
        let ptr = ErasedNonNullPtr::dangling(layout)?;
        let me = unsafe { Self::from_parts(ptr, 0) };
        Ok(me)
    }

    #[inline]
    pub fn from_slice_ptr(ptr: ErasedSlicePtr<ConstPtr<T::Ptrs>>) -> Option<Self> {
        let (ptr, len) = ptr.into_parts();
        let ptr = ErasedNonNullPtr::from_ptr(ptr)?;
        let me = unsafe { Self::from_parts(ptr, len) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_slice_ptr_unchecked(ptr: ErasedSlicePtr<ConstPtr<T::Ptrs>>) -> Self {
        let (ptr, len) = ptr.into_parts();
        let ptr = unsafe { ErasedNonNullPtr::from_ptr_unchecked(ptr) };
        unsafe { Self::from_parts(ptr, len) }
    }

    #[inline]
    pub fn from_mut_slice_ptr(ptr: ErasedMutSlicePtr<MutPtr<T::Ptrs>>) -> Option<Self> {
        let (ptr, len) = ptr.into_parts();
        let ptr = ErasedNonNullPtr::from_mut_ptr(ptr)?;
        let me = unsafe { Self::from_parts(ptr, len) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_mut_slice_ptr_unchecked(ptr: ErasedMutSlicePtr<MutPtr<T::Ptrs>>) -> Self {
        let (ptr, len) = ptr.into_parts();
        let ptr = unsafe { ErasedNonNullPtr::from_mut_ptr_unchecked(ptr) };
        unsafe { Self::from_parts(ptr, len) }
    }

    #[inline]
    pub fn upcast<V>(ptr: NonNull<[V]>) -> Result<Self, UpcastSliceError> {
        let layout = Layout::new::<V>();
        check_ptr_align(ptr.as_ptr().cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let len = ptr.len();
        let slice_len = bytes_to_items::<T::Item>(Layout::array::<V>(len)?.size());
        let slice = NonNull::slice_from_raw_parts(ptr.cast(), slice_len);

        let ptr = unsafe { NonNullSliceItemPtr::from_parts(slice, 0) };
        let ptr = unsafe { ErasedNonNullPtr::from_parts(layout, ptr) };
        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<NonNull<[V]>, DowncastError<Self>> {
        let layout = self.layout();
        let Self { ptr, len, .. } = check_downcast::<V, _>(layout, self)?;

        let data = ptr.as_item_ptr().cast();
        let slice = NonNull::slice_from_raw_parts(data, len);
        Ok(slice)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSlicePtr<ConstPtr<T::Ptrs>> {
        let Self { ptr, len } = self;

        let ptr = ptr.cast_const();
        unsafe { ErasedSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedMutSlicePtr<MutPtr<T::Ptrs>> {
        let Self { ptr, len } = self;

        let ptr = ptr.cast_mut();
        unsafe { ErasedMutSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub fn as_items(self) -> NonNull<[T::Item]> {
        let Self { ptr, len } = self;

        let items = ptr.as_items();
        let len = items.len().wrapping_mul(len);
        NonNull::slice_from_raw_parts(items.cast(), len)
    }

    #[inline]
    pub fn as_item_ptr(self) -> NonNull<T::Item> {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr()
    }
}
