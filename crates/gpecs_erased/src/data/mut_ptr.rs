use core::{alloc::Layout, ptr};

use crate::{
    data::{
        ErasedMutRef, ErasedPtr, ErasedRef,
        error::{DataError, DowncastError, TryFromPtrError, check_downcast},
    },
    error::{InsufficientAlignError, check_len, check_ptr_align, check_sufficient_align},
    layout::bytes_to_items,
    ptr::slice::{CastConst, MutSliceItemPtr},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedMutPtr<T> {
    layout: Layout,
    ptr: T,
}

impl<T> ErasedMutPtr<T> {
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

impl<T> ErasedMutPtr<T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    #[expect(
        clippy::not_unsafe_ptr_arg_deref,
        reason = "`T::from_slice` should not dereference input buffer"
    )]
    pub fn new(layout: Layout, buffer: *mut [T::Item]) -> Result<Self, DataError> {
        check_ptr_align(buffer.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let buffer_layout = Layout::array::<T::Item>(buffer.len())?;
        check_len(buffer_layout.size(), layout.size())?;

        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(layout: Layout) -> Result<Self, InsufficientAlignError> {
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let data = ptr::without_provenance_mut(layout.align());
        let buffer = ptr::slice_from_raw_parts_mut(data, 0);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<*mut V, DowncastError<Self>> {
        let Self { layout, .. } = self;
        let Self { ptr, .. } = check_downcast::<V, _>(layout, self)?;

        let ptr = ptr.as_mut_raw_ptr().cast();
        Ok(ptr)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedPtr<CastConst<T>> {
        let Self { layout, ptr } = self;

        let ptr = ptr.cast_const();
        unsafe { ErasedPtr::from_parts(layout, ptr) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedRef<'a, CastConst<T>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedMutRef<'a, T> {
        unsafe { ErasedMutRef::from_ptr(self) }
    }

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
    pub unsafe fn offset_from(self, origin: ErasedPtr<CastConst<T>>) -> isize {
        let Self { layout, ptr } = self;

        let offset = unsafe { ptr.offset_from(origin.cast_mut().ptr()) };
        let len = bytes_to_items::<T::Item>(layout.size()).cast_signed();
        offset
            .checked_div(len)
            .expect("erased field pointer should not be a ZST")
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
    pub unsafe fn copy_from(self, src: ErasedPtr<CastConst<T>>, count: usize) {
        let Self { layout, ptr } = self;

        let src = src.ptr();
        let count = bytes_to_items::<T::Item>(layout.size()).wrapping_mul(count);
        unsafe { ptr.copy_from(src, count) }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(self, src: ErasedPtr<CastConst<T>>, count: usize) {
        let Self { layout, ptr } = self;

        let src = src.ptr();
        let count = bytes_to_items::<T::Item>(layout.size()).wrapping_mul(count);
        unsafe { ptr.copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    pub fn as_buffer(self) -> *const [T::Item] {
        let Self { layout, ptr } = self;

        let data = ptr.as_mut_raw_ptr().cast_const().cast();
        let len = bytes_to_items::<T::Item>(layout.size());
        ptr::slice_from_raw_parts(data, len)
    }

    #[inline]
    pub fn as_mut_buffer(self) -> *mut [T::Item] {
        let Self { layout, ptr } = self;

        let data = ptr.as_mut_raw_ptr().cast();
        let len = bytes_to_items::<T::Item>(layout.size());
        ptr::slice_from_raw_parts_mut(data, len)
    }

    #[inline]
    pub fn as_ptr(self) -> *const T::Item {
        let Self { ptr, .. } = self;
        ptr.as_mut_raw_ptr().cast_const().cast()
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut T::Item {
        let Self { ptr, .. } = self;
        ptr.as_mut_raw_ptr().cast()
    }
}

impl<T, V> TryFrom<*mut V> for ErasedMutPtr<T>
where
    T: MutSliceItemPtr,
{
    type Error = TryFromPtrError;

    #[inline]
    fn try_from(ptr: *mut V) -> Result<Self, Self::Error> {
        let layout = Layout::new::<V>();
        check_ptr_align(ptr.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let len = bytes_to_items::<T::Item>(layout.size());
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }
}

impl<T, V> TryFrom<ErasedMutPtr<T>> for *mut V
where
    T: MutSliceItemPtr,
{
    type Error = DowncastError<ErasedMutPtr<T>>;

    #[inline]
    fn try_from(ptr: ErasedMutPtr<T>) -> Result<Self, Self::Error> {
        ptr.downcast()
    }
}
