use core::{alloc::Layout, mem::MaybeUninit, ops::Range, ptr::NonNull};

use crate::{
    data::{
        ErasedMutPtr,
        error::{DowncastError, check_downcast},
    },
    error::{InsufficientAlignError, check_sufficient_align},
    layout::bytes_to_items,
    ptr::slice::{MutSliceItemPtr, NonNullAsPtr, NonNullSliceItemPtr},
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

impl<T, U> ErasedNonNullPtr<T>
where
    T: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(ptr: ErasedMutPtr<NonNullAsPtr<T>>) -> Option<Self> {
        let (desc, ptr) = ptr.into_parts();

        let buffer = ptr.slice();
        let buffer = NonNull::new(buffer)?;
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(desc, ptr) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptr: ErasedMutPtr<NonNullAsPtr<T>>) -> Self {
        let (desc, ptr) = ptr.into_parts();

        let buffer = ptr.slice();
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        let ptr = unsafe { T::from_slice(buffer, 0) };

        unsafe { Self::from_parts(desc, ptr) }
    }

    #[inline]
    pub fn dangling(layout: Layout) -> Result<Self, InsufficientAlignError> {
        let ptr = ErasedMutPtr::dangling(layout)?;
        let me = unsafe { Self::new_unchecked(ptr) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<NonNull<V>, DowncastError<Self>> {
        let Self { layout, .. } = self;
        let Self { ptr, .. } = check_downcast::<V, _>(layout, self)?;

        let ptr = ptr.as_item_ptr().cast();
        Ok(ptr)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { layout, ptr } = self;

        let count = bytes_to_items::<U>(layout.size()).wrapping_mul(count);
        let ptr = unsafe { ptr.add(count) };
        unsafe { Self::from_parts(layout, ptr) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { layout, ptr } = self;

        let offset = unsafe { ptr.offset_from(origin.ptr()) };
        let len = bytes_to_items::<U>(layout.size()).cast_signed();
        offset
            .checked_div(len)
            .expect("erased field pointer should not be a ZST")
    }

    #[inline]
    pub unsafe fn swap(self, with: Self) {
        let Self { layout, ptr } = self;

        for i in 0..bytes_to_items::<U>(layout.size()) {
            let this = unsafe { ptr.add(i) }.as_ptr();
            let with = unsafe { with.ptr.add(i) }.as_ptr();
            unsafe { this.swap(with) }
        }
    }

    #[inline]
    pub unsafe fn copy_from(self, src: Self, count: usize) {
        let Self { layout, ptr } = self;

        let src = src.ptr().as_ptr().cast_const();
        let count = bytes_to_items::<U>(layout.size()).wrapping_mul(count);
        unsafe { ptr.as_ptr().copy_from(src, count) }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(self, src: Self, count: usize) {
        let Self { layout, ptr } = self;

        let src = src.ptr().as_ptr().cast_const();
        let count = bytes_to_items::<U>(layout.size()).wrapping_mul(count);
        unsafe { ptr.as_ptr().copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> NonNull<[MaybeUninit<U>]> {
        let Self { ptr, .. } = self;
        ptr.slice()
    }

    #[inline]
    pub fn byte_offset(self) -> usize {
        let Self { ptr, .. } = self;
        ptr.index().wrapping_mul(size_of::<U>())
    }

    #[inline]
    pub fn buffer_init_range(self) -> Range<usize> {
        let Self { layout, ptr } = self;

        let start = ptr.index();
        let end = start + bytes_to_items::<U>(layout.size());
        start..end
    }

    #[inline]
    pub fn as_buffer(self) -> NonNull<[U]> {
        let Self { layout, ptr } = self;

        let data = ptr.as_item_ptr().cast();
        let len = bytes_to_items::<U>(layout.size());
        NonNull::slice_from_raw_parts(data, len)
    }

    #[inline]
    pub fn as_ptr(self) -> NonNull<U> {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr().cast()
    }
}

impl<T, U, V> TryFrom<NonNull<V>> for ErasedNonNullPtr<T>
where
    T: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: NonNull<V>) -> Result<Self, Self::Error> {
        let layout = Layout::new::<V>();
        check_sufficient_align(layout, Layout::new::<U>())?;

        let len = bytes_to_items::<U>(layout.size());
        let buffer = NonNull::slice_from_raw_parts(ptr.cast(), len);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }
}

impl<T, U, V> TryFrom<ErasedNonNullPtr<T>> for NonNull<V>
where
    T: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = DowncastError<ErasedNonNullPtr<T>>;

    #[inline]
    fn try_from(ptr: ErasedNonNullPtr<T>) -> Result<Self, Self::Error> {
        ptr.downcast()
    }
}
