use core::{alloc::Layout, num::NonZeroUsize, ptr::NonNull};

use crate::{
    data::{
        ErasedMutPtr, ErasedMutRef, ErasedPtr, ErasedRef,
        error::{DataError, DowncastError, UpcastPtrError, check_downcast},
    },
    error::{InsufficientAlignError, check_len, check_ptr_align, check_sufficient_align},
    layout::bytes_to_items,
    ptr::slice::{
        ConstPtr, ConstSliceItemPtr, MutPtr, MutSliceItemPtr, NonNullSliceItemPtr, SliceItemPtr,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedNonNullPtr<T> {
    layout: Layout,
    ptr: T,
}

impl<T> ErasedNonNullPtr<T> {
    #[inline]
    pub unsafe fn from_parts(layout: Layout, ptr: T) -> Self {
        Self { layout, ptr }
    }

    #[inline]
    pub fn layout(self) -> Layout {
        let Self { layout, .. } = self;
        layout
    }

    #[inline]
    pub fn ptr(self) -> T {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, T) {
        let Self { layout, ptr } = self;
        (layout, ptr)
    }
}

impl<T> ErasedNonNullPtr<T>
where
    T: SliceItemPtr,
{
    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { layout, ptr } = self;

        let count = bytes_to_items::<T::Item>(layout.size()).wrapping_mul(count);
        let ptr = unsafe { ptr.add(count) };
        unsafe { Self::from_parts(layout, ptr) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { layout, ptr } = self;

        let offset = unsafe { ptr.offset_from(origin.ptr()) };
        let len = bytes_to_items::<T::Item>(layout.size()).cast_signed();
        offset
            .checked_div(len)
            .expect("erased field pointer should not be a ZST")
    }

    #[inline]
    pub unsafe fn as_ref<'a>(self) -> ErasedRef<'a, T> {
        unsafe { ErasedRef::from_nonnull_ptr(self) }
    }

    #[inline]
    pub unsafe fn as_mut<'a>(self) -> ErasedMutRef<'a, T> {
        unsafe { ErasedMutRef::from_nonnull_ptr(self) }
    }
}

impl<T> ErasedNonNullPtr<T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub fn new(layout: Layout, items: NonNull<[T::Item]>) -> Result<Self, DataError> {
        check_ptr_align(items.as_ptr().cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let layout_of_items = Layout::array::<T::Item>(items.len())?;
        check_len(layout_of_items.size(), layout.size())?;

        let ptr = unsafe { T::from_parts(items, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(layout: Layout) -> Result<Self, InsufficientAlignError> {
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let addr = unsafe { NonZeroUsize::new_unchecked(layout.align()) };
        let data = NonNull::without_provenance(addr);
        let slice = NonNull::slice_from_raw_parts(data, 0);
        let ptr = unsafe { T::from_parts(slice, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }

    #[inline]
    pub fn from_ptr(ptr: ErasedPtr<ConstPtr<T::Ptrs>>) -> Option<Self> {
        let (layout, ptr) = ptr.into_parts();
        let ptr = ptr.cast_nonnull()?;
        let me = unsafe { Self::from_parts(layout, ptr) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_ptr_unchecked(ptr: ErasedPtr<ConstPtr<T::Ptrs>>) -> Self {
        let (layout, ptr) = ptr.into_parts();
        let ptr = unsafe { ptr.cast_nonnull_unchecked() };
        unsafe { Self::from_parts(layout, ptr) }
    }

    #[inline]
    pub fn from_mut_ptr(ptr: ErasedMutPtr<MutPtr<T::Ptrs>>) -> Option<Self> {
        let (layout, ptr) = ptr.into_parts();
        let ptr = ptr.cast_nonnull()?;
        let me = unsafe { Self::from_parts(layout, ptr) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_mut_ptr_unchecked(ptr: ErasedMutPtr<MutPtr<T::Ptrs>>) -> Self {
        let (layout, ptr) = ptr.into_parts();
        let ptr = unsafe { ptr.cast_nonnull_unchecked() };
        unsafe { Self::from_parts(layout, ptr) }
    }

    #[inline]
    pub fn upcast<V>(ptr: NonNull<V>) -> Result<Self, UpcastPtrError> {
        let layout = Layout::new::<V>();
        check_ptr_align(ptr.as_ptr().cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let len = bytes_to_items::<T::Item>(layout.size());
        let slice = NonNull::slice_from_raw_parts(ptr.cast(), len);
        let ptr = unsafe { T::from_parts(slice, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<NonNull<V>, DowncastError<Self>> {
        let Self { layout, .. } = self;
        let Self { ptr, .. } = check_downcast::<V, _>(layout, self)?;

        let ptr = ptr.as_ptr().cast();
        Ok(ptr)
    }

    #[inline]
    pub unsafe fn swap(self, with: Self) {
        let Self { layout, ptr } = self;

        for i in 0..bytes_to_items::<T::Item>(layout.size()) {
            let this = unsafe { ptr.add(i) };
            let with = unsafe { with.ptr.add(i) };
            unsafe { this.swap(with) }
        }
    }

    #[inline]
    pub unsafe fn swap_nonoverlapping(self, with: Self, count: usize) {
        let Self { layout, ptr } = self;

        for i in 0..bytes_to_items::<T::Item>(layout.size()) {
            let this = unsafe { ptr.add(i) };
            let with = unsafe { with.ptr.add(i) };
            unsafe { this.swap_nonoverlapping(with, count) }
        }
    }

    #[inline]
    pub unsafe fn copy_from(self, src: Self, count: usize) {
        let Self { layout, ptr } = self;

        let src = src.ptr();
        let count = bytes_to_items::<T::Item>(layout.size()).wrapping_mul(count);
        unsafe { ptr.copy_from(src, count) }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(self, src: Self, count: usize) {
        let Self { layout, ptr } = self;

        let src = src.ptr();
        let count = bytes_to_items::<T::Item>(layout.size()).wrapping_mul(count);
        unsafe { ptr.copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    pub fn as_items(self) -> NonNull<[T::Item]> {
        let Self { layout, ptr } = self;

        let data = ptr.as_ptr();
        let len = bytes_to_items::<T::Item>(layout.size());
        NonNull::slice_from_raw_parts(data, len)
    }

    #[inline]
    pub fn as_item_ptr(self) -> NonNull<T::Item> {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn cast_const(self) -> ErasedPtr<ConstPtr<T::Ptrs>> {
        let Self { layout, ptr } = self;

        let ptr = ptr.cast_const();
        unsafe { ErasedPtr::from_parts(layout, ptr) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedMutPtr<MutPtr<T::Ptrs>> {
        let Self { layout, ptr } = self;

        let ptr = ptr.cast_mut();
        unsafe { ErasedMutPtr::from_parts(layout, ptr) }
    }
}
