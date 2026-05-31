use core::{
    marker::PhantomData,
    ptr::{self, NonNull},
};
use spirv_std::arch::IndexUnchecked;

use gpecs_soa_erased::ptr::slice::{
    ConstSliceItemPtr, MutSliceItemPtr, NonNullSliceItemPtr, SliceItemPtr, SliceItemPtrs,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuSliceItemPtrs<T> {
    phantom: PhantomData<T>,
}

unsafe impl<T> SliceItemPtrs for GpuSliceItemPtrs<T>
where
    T: Copy,
{
    type Item = T;

    type Const = GpuSliceItemPtr<*const [T]>;
    type Mut = GpuSliceItemPtr<*mut [T]>;
    type NonNull = GpuSliceItemPtr<NonNull<[T]>>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct GpuSliceItemPtr<T>
where
    T: ?Sized,
{
    pub index: usize,
    pub slice: T,
}

impl<T> GpuSliceItemPtr<T>
where
    T: ?Sized,
{
    #[inline]
    pub unsafe fn offset_from(&self, origin: &Self) -> isize {
        let index = self.index.cast_signed();
        let origin_index = origin.index.cast_signed();
        index.wrapping_sub(origin_index)
    }
}

impl<T> GpuSliceItemPtr<T> {
    #[inline]
    #[must_use = "returns a new pointer rather than modifying its argument"]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { index, .. } = self;
        let index = unsafe { index.unchecked_add(count) };
        Self { index, ..self }
    }
}

unsafe impl<T> SliceItemPtr for GpuSliceItemPtr<*const [T]>
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn index(self) -> usize {
        self.index
    }

    #[inline]
    unsafe fn add(self, count: usize) -> Self {
        unsafe { GpuSliceItemPtr::add(self, count) }
    }

    #[inline]
    unsafe fn offset_from(self, origin: Self) -> isize {
        unsafe { GpuSliceItemPtr::offset_from(&self, &origin) }
    }

    #[inline]
    unsafe fn read(self) -> T {
        unsafe { *self.as_ref_unchecked() }
    }
}

unsafe impl<T> ConstSliceItemPtr for GpuSliceItemPtr<*const [T]>
where
    T: Copy,
{
    type Ptrs = GpuSliceItemPtrs<T>;

    #[inline]
    unsafe fn from_slice(slice: *const [T], index: usize) -> Self {
        Self { index, slice }
    }

    #[inline]
    fn slice(self) -> *const [T] {
        self.slice
    }

    #[inline]
    fn as_raw_ptr(self) -> *const Self::Item {
        let Self { index, slice } = self;
        let item = unsafe { (*slice).index_unchecked(index) };
        ptr::from_ref(item)
    }
}

unsafe impl<T> SliceItemPtr for GpuSliceItemPtr<*mut [T]>
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn index(self) -> usize {
        self.index
    }

    #[inline]
    unsafe fn add(self, count: usize) -> Self {
        unsafe { GpuSliceItemPtr::add(self, count) }
    }

    #[inline]
    unsafe fn offset_from(self, origin: Self) -> isize {
        unsafe { GpuSliceItemPtr::offset_from(&self, &origin) }
    }

    #[inline]
    unsafe fn read(self) -> T {
        unsafe { *self.as_mut_unchecked() }
    }
}

unsafe impl<T> MutSliceItemPtr for GpuSliceItemPtr<*mut [T]>
where
    T: Copy,
{
    type Ptrs = GpuSliceItemPtrs<T>;

    #[inline]
    unsafe fn from_slice(slice: *mut [T], index: usize) -> Self {
        Self { index, slice }
    }

    #[inline]
    fn slice(self) -> *mut [T] {
        self.slice
    }

    #[inline]
    fn as_mut_raw_ptr(self) -> *mut T {
        let Self { index, slice } = self;
        let item = unsafe { (*slice).index_unchecked_mut(index) };
        ptr::from_mut(item)
    }

    #[inline]
    unsafe fn write(self, value: T) {
        let dst = unsafe { self.as_mut_unchecked() };
        *dst = value;
    }

    #[inline]
    unsafe fn swap(self, with: Self) {
        let tmp = unsafe { self.read() };
        unsafe { self.write(with.read()) }
        unsafe { with.write(tmp) }
    }

    #[inline]
    unsafe fn copy_from(self, src: GpuSliceItemPtr<*const [T]>, count: usize) {
        // Assuming slices do not overlap or are pointing to the same memory region
        // because they can only be obtained from storage buffers / shared memory on Rust-GPU backend...
        // And if they do overlap, this is a UB by Rust definitions even when using safe code!
        if self.index <= src.index {
            // Copy forwards, as `self` starts before `src`
            unsafe { ordered_copy(src, self, 0..count) }
        } else {
            // Copy backwards, as `self` starts after `src`
            unsafe { ordered_copy(src, self, (0..count).rev()) }
        }
    }

    #[inline]
    unsafe fn copy_from_nonoverlapping(self, src: GpuSliceItemPtr<*const [T]>, count: usize) {
        // Always copy forwards for non-overlapping slices
        unsafe { ordered_copy(src, self, 0..count) }
    }
}

#[inline]
unsafe fn ordered_copy<T, S, D, I>(src: S, dst: D, indices: I)
where
    S: ConstSliceItemPtr<Item = T>,
    D: MutSliceItemPtr<Item = T>,
    I: IntoIterator<Item = usize>,
{
    for i in indices {
        let src = unsafe { src.add(i).read() };
        unsafe { dst.add(i).write(src) }
    }
}

unsafe impl<T> SliceItemPtr for GpuSliceItemPtr<NonNull<[T]>>
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn index(self) -> usize {
        self.index
    }

    #[inline]
    unsafe fn add(self, count: usize) -> Self {
        unsafe { GpuSliceItemPtr::add(self, count) }
    }

    #[inline]
    unsafe fn offset_from(self, origin: Self) -> isize {
        unsafe { GpuSliceItemPtr::offset_from(&self, &origin) }
    }

    #[inline]
    unsafe fn read(self) -> T {
        unsafe { *self.as_ptr().as_mut_unchecked() }
    }
}

unsafe impl<T> NonNullSliceItemPtr for GpuSliceItemPtr<NonNull<[T]>>
where
    T: Copy,
{
    type Ptrs = GpuSliceItemPtrs<T>;

    #[inline]
    unsafe fn from_slice(slice: NonNull<[T]>, index: usize) -> Self {
        Self { index, slice }
    }

    #[inline]
    fn slice(self) -> NonNull<[T]> {
        self.slice
    }

    #[inline]
    fn as_raw_ptr(self) -> NonNull<Self::Item> {
        let Self { index, slice } = self;
        let item = unsafe { slice.as_ref().index_unchecked(index) };
        NonNull::from_ref(item)
    }
}
