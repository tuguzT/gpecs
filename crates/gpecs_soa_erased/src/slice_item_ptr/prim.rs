use core::ptr::{self, NonNull};

use crate::slice_item_ptr::{
    ConstSliceItemPtr, MutSliceItemPtr, NonNullSliceItemPtr, SliceItemPtr, SliceItemPtrs,
};

unsafe impl<T> SliceItemPtrs<T> for () {
    type Const = *const T;
    type Mut = *mut T;
    type NonNull = NonNull<T>;
}

unsafe impl<T> SliceItemPtr for *const T {
    type Item = T;

    #[inline]
    fn index(self) -> usize {
        0
    }

    #[inline]
    unsafe fn add(self, count: usize) -> Self {
        unsafe { self.add(count) }
    }

    #[inline]
    unsafe fn offset_from(self, origin: Self) -> isize {
        unsafe { self.offset_from(origin) }
    }

    #[inline]
    unsafe fn read(self) -> T {
        unsafe { self.read() }
    }
}

unsafe impl<T> ConstSliceItemPtr for *const T {
    type Ptrs = ();

    #[inline]
    unsafe fn from_slice(slice: *const [T], index: usize) -> Self {
        unsafe { slice.cast::<T>().add(index) }
    }

    #[inline]
    fn slice(self) -> *const [T] {
        ptr::slice_from_raw_parts(self, 1)
    }

    #[inline]
    unsafe fn as_ref<'a>(self) -> &'a T {
        unsafe { &*self }
    }
}

unsafe impl<T> SliceItemPtr for *mut T {
    type Item = T;

    #[inline]
    fn index(self) -> usize {
        0
    }

    #[inline]
    unsafe fn add(self, count: usize) -> Self {
        unsafe { self.add(count) }
    }

    #[inline]
    unsafe fn offset_from(self, origin: Self) -> isize {
        unsafe { self.offset_from(origin) }
    }

    #[inline]
    unsafe fn read(self) -> T {
        unsafe { self.read() }
    }
}

unsafe impl<T> MutSliceItemPtr for *mut T {
    type Ptrs = ();

    #[inline]
    unsafe fn from_slice(slice: *mut [T], index: usize) -> Self {
        unsafe { slice.cast::<T>().add(index) }
    }

    #[inline]
    fn slice(self) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(self, 1)
    }

    #[inline]
    unsafe fn as_mut<'a>(self) -> &'a mut T {
        unsafe { &mut *self }
    }

    #[inline]
    unsafe fn write(self, value: T) {
        unsafe { self.write(value) }
    }

    #[inline]
    unsafe fn swap(self, with: Self) {
        unsafe { self.swap(with) }
    }

    #[inline]
    unsafe fn copy_from(self, src: *const T, count: usize) {
        unsafe { self.copy_from(src, count) }
    }

    #[inline]
    unsafe fn copy_from_nonoverlapping(self, src: *const T, count: usize) {
        unsafe { self.copy_from_nonoverlapping(src, count) }
    }
}

unsafe impl<T> SliceItemPtr for NonNull<T> {
    type Item = T;

    #[inline]
    fn index(self) -> usize {
        0
    }

    #[inline]
    unsafe fn add(self, count: usize) -> Self {
        unsafe { self.add(count) }
    }

    #[inline]
    unsafe fn offset_from(self, origin: Self) -> isize {
        unsafe { self.offset_from(origin) }
    }

    #[inline]
    unsafe fn read(self) -> T {
        unsafe { self.read() }
    }
}

unsafe impl<T> NonNullSliceItemPtr for NonNull<T> {
    type Ptrs = ();

    #[inline]
    unsafe fn from_slice(slice: NonNull<[T]>, index: usize) -> Self {
        unsafe { slice.cast::<T>().add(index) }
    }

    #[inline]
    fn slice(self) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(self, 1)
    }
}
