use core::{
    any, cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::slice::{
    ConstSlicePtrAtIndex, MutSlicePtrAtIndex, NonNullSlicePtrAtIndex, SlicePtrAtIndex,
    SlicePtrsAtIndex,
};

pub struct CoreSlicePtrsAtIndex<T>(PhantomData<T>);

impl<T> CoreSlicePtrsAtIndex<T> {
    #[inline]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Debug for CoreSlicePtrsAtIndex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = any::type_name::<T>();
        write!(f, "CoreSlicePtrsAtIndex<{type_name}>")
    }
}

impl<T> Default for CoreSlicePtrsAtIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for CoreSlicePtrsAtIndex<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for CoreSlicePtrsAtIndex<T> {}

impl<T> PartialEq for CoreSlicePtrsAtIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for CoreSlicePtrsAtIndex<T> {}

impl<T> PartialOrd for CoreSlicePtrsAtIndex<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for CoreSlicePtrsAtIndex<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for CoreSlicePtrsAtIndex<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

unsafe impl<T> SlicePtrsAtIndex for CoreSlicePtrsAtIndex<T> {
    type Item = T;

    type Const = *const T;
    type Mut = *mut T;
    type NonNull = NonNull<T>;
}

unsafe impl<T> SlicePtrAtIndex for *const T {
    type Item = T;

    #[inline]
    fn dangling() -> Self {
        ptr::dangling()
    }

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

unsafe impl<T> ConstSlicePtrAtIndex for *const T {
    type Ptrs = CoreSlicePtrsAtIndex<T>;

    #[inline]
    unsafe fn from_parts(slice: *const [T], index: usize) -> Self {
        unsafe { slice.cast::<T>().add(index) }
    }

    #[inline]
    fn slice(self) -> *const [T] {
        ptr::slice_from_raw_parts(self, 0)
    }

    #[inline]
    fn as_ptr(self) -> *const Self::Item {
        self
    }
}

unsafe impl<T> SlicePtrAtIndex for *mut T {
    type Item = T;

    #[inline]
    fn dangling() -> Self {
        ptr::dangling_mut()
    }

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

unsafe impl<T> MutSlicePtrAtIndex for *mut T {
    type Ptrs = CoreSlicePtrsAtIndex<T>;

    #[inline]
    unsafe fn from_parts(slice: *mut [T], index: usize) -> Self {
        unsafe { slice.cast::<T>().add(index) }
    }

    #[inline]
    fn slice(self) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(self, 0)
    }

    #[inline]
    fn as_mut_ptr(self) -> *mut Self::Item {
        self
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
    unsafe fn swap_nonoverlapping(self, with: Self, count: usize) {
        unsafe { ptr::swap_nonoverlapping(self, with, count) }
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

unsafe impl<T> SlicePtrAtIndex for NonNull<T> {
    type Item = T;

    #[inline]
    fn dangling() -> Self {
        NonNull::dangling()
    }

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

unsafe impl<T> NonNullSlicePtrAtIndex for NonNull<T> {
    type Ptrs = CoreSlicePtrsAtIndex<T>;

    #[inline]
    unsafe fn from_parts(slice: NonNull<[T]>, index: usize) -> Self {
        unsafe { slice.cast::<T>().add(index) }
    }

    #[inline]
    fn slice(self) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(self, 0)
    }

    #[inline]
    fn from_ref(r: &Self::Item) -> Self {
        NonNull::from_ref(r)
    }

    #[inline]
    fn from_mut(r: &mut Self::Item) -> Self {
        NonNull::from_mut(r)
    }

    #[inline]
    fn as_ptr(self) -> NonNull<Self::Item> {
        self
    }
}
