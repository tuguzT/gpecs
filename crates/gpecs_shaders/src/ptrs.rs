use core::{
    any, cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr::{self, NonNull},
};
use spirv_std::arch::IndexUnchecked;

use gpecs_soa_erased::ptr::slice::{
    ConstSlicePtrAtIndex, MutSlicePtrAtIndex, NonNullSlicePtrAtIndex, SlicePtrAtIndex,
    SlicePtrsAtIndex,
};

pub struct GpuSlicePtrsAtIndex<T>(PhantomData<T>);

impl<T> GpuSlicePtrsAtIndex<T> {
    #[inline]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Debug for GpuSlicePtrsAtIndex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = any::type_name::<T>();
        write!(f, "GpuSliceItemPtrs<{type_name}>")
    }
}

impl<T> Default for GpuSlicePtrsAtIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for GpuSlicePtrsAtIndex<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for GpuSlicePtrsAtIndex<T> {}

impl<T> PartialEq for GpuSlicePtrsAtIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for GpuSlicePtrsAtIndex<T> {}

impl<T> PartialOrd for GpuSlicePtrsAtIndex<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for GpuSlicePtrsAtIndex<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for GpuSlicePtrsAtIndex<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

unsafe impl<T> SlicePtrsAtIndex for GpuSlicePtrsAtIndex<T>
where
    T: Copy,
{
    type Item = T;

    type Const = GpuSlicePtrAtIndex<*const [T]>;
    type Mut = GpuSlicePtrAtIndex<*mut [T]>;
    type NonNull = GpuSlicePtrAtIndex<NonNull<[T]>>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct GpuSlicePtrAtIndex<T>
where
    T: ?Sized,
{
    pub index: usize,
    pub slice: T,
}

impl<T> GpuSlicePtrAtIndex<T>
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

impl<T> GpuSlicePtrAtIndex<T> {
    #[inline]
    #[must_use = "returns a new pointer rather than modifying its argument"]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { index, .. } = self;
        let index = unsafe { index.unchecked_add(count) };
        Self { index, ..self }
    }
}

unsafe impl<T> SlicePtrAtIndex for GpuSlicePtrAtIndex<*const [T]>
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn dangling() -> Self {
        let data = ptr::dangling();
        let slice = ptr::slice_from_raw_parts(data, 0);
        Self { index: 0, slice }
    }

    #[inline]
    fn index(self) -> usize {
        self.index
    }

    #[inline]
    unsafe fn add(self, count: usize) -> Self {
        unsafe { GpuSlicePtrAtIndex::add(self, count) }
    }

    #[inline]
    unsafe fn offset_from(self, origin: Self) -> isize {
        unsafe { GpuSlicePtrAtIndex::offset_from(&self, &origin) }
    }

    #[inline]
    unsafe fn read(self) -> T {
        unsafe { *self.as_ref_unchecked() }
    }
}

unsafe impl<T> ConstSlicePtrAtIndex for GpuSlicePtrAtIndex<*const [T]>
where
    T: Copy,
{
    type Ptrs = GpuSlicePtrsAtIndex<T>;

    #[inline]
    unsafe fn from_parts(slice: *const [T], index: usize) -> Self {
        Self { index, slice }
    }

    #[inline]
    fn slice(self) -> *const [T] {
        self.slice
    }

    #[inline]
    fn as_ptr(self) -> *const Self::Item {
        let Self { index, slice } = self;
        let item = unsafe { (*slice).index_unchecked(index) };
        ptr::from_ref(item)
    }
}

unsafe impl<T> SlicePtrAtIndex for GpuSlicePtrAtIndex<*mut [T]>
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn dangling() -> Self {
        let data = ptr::dangling_mut();
        let slice = ptr::slice_from_raw_parts_mut(data, 0);
        Self { index: 0, slice }
    }

    #[inline]
    fn index(self) -> usize {
        self.index
    }

    #[inline]
    unsafe fn add(self, count: usize) -> Self {
        unsafe { GpuSlicePtrAtIndex::add(self, count) }
    }

    #[inline]
    unsafe fn offset_from(self, origin: Self) -> isize {
        unsafe { GpuSlicePtrAtIndex::offset_from(&self, &origin) }
    }

    #[inline]
    unsafe fn read(self) -> T {
        unsafe { *self.as_mut_unchecked() }
    }
}

unsafe impl<T> MutSlicePtrAtIndex for GpuSlicePtrAtIndex<*mut [T]>
where
    T: Copy,
{
    type Ptrs = GpuSlicePtrsAtIndex<T>;

    #[inline]
    unsafe fn from_parts(slice: *mut [T], index: usize) -> Self {
        Self { index, slice }
    }

    #[inline]
    fn slice(self) -> *mut [T] {
        self.slice
    }

    #[inline]
    fn as_mut_ptr(self) -> *mut T {
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
    unsafe fn swap_nonoverlapping(self, with: Self, count: usize) {
        for i in 0..count {
            let this = unsafe { self.add(i) };
            let with = unsafe { with.add(i) };
            unsafe { this.swap(with) }
        }
    }

    #[inline]
    unsafe fn copy_from(self, src: GpuSlicePtrAtIndex<*const [T]>, count: usize) {
        // Assuming slices do not overlap or are pointing to the same memory region
        // because they can only be obtained from storage buffers / shared memory on Rust-GPU backend...
        // And if they do overlap, this is an UB by Rust definitions even when using safe code!
        if self.index <= src.index {
            // Copy forwards, as `self` starts before `src`
            unsafe { ordered_copy(src, self, 0..count) }
        } else {
            // Copy backwards, as `self` starts after `src`
            unsafe { ordered_copy(src, self, (0..count).rev()) }
        }
    }

    #[inline]
    unsafe fn copy_from_nonoverlapping(self, src: GpuSlicePtrAtIndex<*const [T]>, count: usize) {
        // Always copy forwards for non-overlapping slices
        unsafe { ordered_copy(src, self, 0..count) }
    }
}

#[inline]
unsafe fn ordered_copy<T, S, D, I>(src: S, dst: D, indices: I)
where
    S: ConstSlicePtrAtIndex<Item = T>,
    D: MutSlicePtrAtIndex<Item = T>,
    I: IntoIterator<Item = usize>,
{
    for i in indices {
        let src = unsafe { src.add(i).read() };
        unsafe { dst.add(i).write(src) }
    }
}

unsafe impl<T> SlicePtrAtIndex for GpuSlicePtrAtIndex<NonNull<[T]>>
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn dangling() -> Self {
        let data = NonNull::dangling();
        let slice = NonNull::slice_from_raw_parts(data, 0);
        Self { index: 0, slice }
    }

    #[inline]
    fn index(self) -> usize {
        self.index
    }

    #[inline]
    unsafe fn add(self, count: usize) -> Self {
        unsafe { GpuSlicePtrAtIndex::add(self, count) }
    }

    #[inline]
    unsafe fn offset_from(self, origin: Self) -> isize {
        unsafe { GpuSlicePtrAtIndex::offset_from(&self, &origin) }
    }

    #[inline]
    unsafe fn read(self) -> T {
        unsafe { self.cast_const().read() }
    }
}

unsafe impl<T> NonNullSlicePtrAtIndex for GpuSlicePtrAtIndex<NonNull<[T]>>
where
    T: Copy,
{
    type Ptrs = GpuSlicePtrsAtIndex<T>;

    #[inline]
    unsafe fn from_parts(slice: NonNull<[T]>, index: usize) -> Self {
        Self { index, slice }
    }

    #[inline]
    fn slice(self) -> NonNull<[T]> {
        self.slice
    }

    #[inline]
    fn as_ptr(self) -> NonNull<Self::Item> {
        let Self { index, slice } = self;
        let item = unsafe { slice.as_ref().index_unchecked(index) };
        NonNull::from_ref(item)
    }
}
