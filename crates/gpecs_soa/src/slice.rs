use alloc::vec::Vec;
use core::{
    cmp,
    fmt::{self, Debug},
    marker::PhantomData,
    ops,
    ptr::{self, NonNull},
    slice,
};

use crate::ptr::{
    min_size_of, ptrs, slice_from_len_in_bytes, slice_from_len_in_bytes_mut, slice_from_raw_parts,
    slice_from_raw_parts_mut, to_len,
};

#[repr(transparent)]
pub struct SoaSlice<T, U, V> {
    phantom: PhantomData<(T, U, V)>,
    buffer: [u8],
}

impl<T, U, V> SoaSlice<T, U, V> {
    #[inline]
    pub const fn len(&self) -> usize {
        match self.buffer_capacity() {
            0 => 0,
            _ => unsafe { ptr::read(self.as_ptr().cast()) },
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn buffer_capacity(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        if min_size_of::<T, U, V>() == 0 {
            usize::MAX
        } else {
            let len_in_bytes = self.buffer_capacity();
            to_len::<T, U, V>(len_in_bytes)
        }
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const T, *const U, *const V) {
        let ptr = self.as_ptr().cast_mut();
        let len = self.capacity();

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut T, *mut U, *mut V) {
        let ptr = self.as_mut_ptr();
        let len = self.capacity();

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_slices(&self) -> (&[T], &[U], &[V]) {
        let (t_data, u_data, v_data) = self.as_ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts(t_data, len);
            let u_slice = slice::from_raw_parts(u_data, len);
            let v_slice = slice::from_raw_parts(v_data, len);
            (t_slice, u_slice, v_slice)
        }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [U], &mut [V]) {
        let (t_data, u_data, v_data) = self.as_mut_ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts_mut(t_data, len);
            let u_slice = slice::from_raw_parts_mut(u_data, len);
            let v_slice = slice::from_raw_parts_mut(v_data, len);
            (t_slice, u_slice, v_slice)
        }
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Ref<'_>>
    where
        I: SoaSliceIndex<Self>,
    {
        index.get(self)
    }

    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::RefMut<'_>>
    where
        I: SoaSliceIndex<Self>,
    {
        index.get_mut(self)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::ConstPtr
    where
        I: SoaSliceIndex<Self>,
    {
        unsafe { index.get_unchecked(self) }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtr
    where
        I: SoaSliceIndex<Self>,
    {
        unsafe { index.get_unchecked_mut(self) }
    }

    #[inline]
    pub fn index<I>(&self, index: I) -> I::Ref<'_>
    where
        I: SoaSliceIndex<Self>,
    {
        index.index(self)
    }

    #[inline]
    pub fn index_mut<I>(&mut self, index: I) -> I::RefMut<'_>
    where
        I: SoaSliceIndex<Self>,
    {
        index.index_mut(self)
    }

    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: usize, b: usize) {
        let (t_a, u_a, v_a) = {
            let (t, u, v) = self.index_mut(a);
            (t as _, u as _, v as _)
        };
        let (t_b, u_b, v_b) = {
            let (t, u, v) = self.index_mut(b);
            (t as _, u as _, v as _)
        };

        unsafe {
            ptr::swap(t_a, t_b);
            ptr::swap(u_a, u_b);
            ptr::swap(v_a, v_b);
        }
    }

    #[inline]
    pub fn sort(&mut self)
    where
        T: Ord,
        U: Ord,
        V: Ord,
    {
        self.sort_by(|a, b| a.cmp(&b))
    }

    #[inline]
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut((&T, &U, &V), (&T, &U, &V)) -> cmp::Ordering,
    {
        let (t_ptr, u_ptr, v_ptr) = self.as_ptrs();
        self.sort_impl(|indices| {
            indices.sort_by(|&a, &b| {
                let a = unsafe {
                    let t_ptr = t_ptr.add(a);
                    let u_ptr = u_ptr.add(a);
                    let v_ptr = v_ptr.add(a);
                    (&*t_ptr, &*u_ptr, &*v_ptr)
                };
                let b = unsafe {
                    let t_ptr = t_ptr.add(b);
                    let u_ptr = u_ptr.add(b);
                    let v_ptr = v_ptr.add(b);
                    (&*t_ptr, &*u_ptr, &*v_ptr)
                };
                compare(a, b)
            })
        })
    }

    #[inline]
    pub fn sort_by_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut((&T, &U, &V)) -> K,
        K: Ord,
    {
        let (t_ptr, u_ptr, v_ptr) = self.as_ptrs();
        self.sort_impl(|indices| {
            indices.sort_by_key(|&index| unsafe {
                let t_ptr = t_ptr.add(index);
                let u_ptr = u_ptr.add(index);
                let v_ptr = v_ptr.add(index);
                f((&*t_ptr, &*u_ptr, &*v_ptr))
            })
        })
    }

    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut((&T, &U, &V)) -> K,
        K: Ord,
    {
        let (t_ptr, u_ptr, v_ptr) = self.as_ptrs();
        self.sort_impl(|indices| {
            indices.sort_by_cached_key(|&index| unsafe {
                let t_ptr = t_ptr.add(index);
                let u_ptr = u_ptr.add(index);
                let v_ptr = v_ptr.add(index);
                f((&*t_ptr, &*u_ptr, &*v_ptr))
            })
        })
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        T: Ord,
        U: Ord,
        V: Ord,
    {
        self.sort_unstable_by(|a, b| a.cmp(&b))
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, mut compare: F)
    where
        F: FnMut((&T, &U, &V), (&T, &U, &V)) -> cmp::Ordering,
    {
        let (t_ptr, u_ptr, v_ptr) = self.as_ptrs();
        self.sort_impl(|indices| {
            indices.sort_unstable_by(|&a, &b| {
                let a = unsafe {
                    let t_ptr = t_ptr.add(a);
                    let u_ptr = u_ptr.add(a);
                    let v_ptr = v_ptr.add(a);
                    (&*t_ptr, &*u_ptr, &*v_ptr)
                };
                let b = unsafe {
                    let t_ptr = t_ptr.add(b);
                    let u_ptr = u_ptr.add(b);
                    let v_ptr = v_ptr.add(b);
                    (&*t_ptr, &*u_ptr, &*v_ptr)
                };
                compare(a, b)
            })
        })
    }

    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut((&T, &U, &V)) -> K,
        K: Ord,
    {
        let (t_ptr, u_ptr, v_ptr) = self.as_ptrs();
        self.sort_impl(|indices| {
            indices.sort_unstable_by_key(|&index| unsafe {
                let t_ptr = t_ptr.add(index);
                let u_ptr = u_ptr.add(index);
                let v_ptr = v_ptr.add(index);
                f((&*t_ptr, &*u_ptr, &*v_ptr))
            })
        })
    }

    fn sort_impl<F>(&mut self, f: F)
    where
        F: FnOnce(&mut [usize]),
    {
        let len = self.len();
        if min_size_of::<T, U, V>() == 0 || len < 2 {
            return;
        }

        let mut permutation: Vec<_> = (0..len).collect();
        f(&mut permutation);

        for src in 0..len {
            let dst = permutation[src];
            if src == dst {
                continue;
            }
            self.swap(src, dst);
            permutation.swap(src, dst);
        }
    }
}

impl<T, U, V> Debug for SoaSlice<T, U, V>
where
    T: Debug,
    U: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t_slice, u_slice, v_slice) = self.as_slices();
        f.debug_struct("MultiSlice")
            .field("t_slice", &t_slice)
            .field("u_slice", &u_slice)
            .field("v_slice", &v_slice)
            .finish()
    }
}

impl<T, U, V> Default for &SoaSlice<T, U, V> {
    fn default() -> Self {
        let data = NonNull::dangling().as_ptr();
        unsafe { from_len_in_bytes(data, 0) }
    }
}

impl<T, U, V> Default for &mut SoaSlice<T, U, V> {
    fn default() -> Self {
        let data = NonNull::dangling().as_ptr();
        unsafe { from_len_in_bytes_mut(data, 0) }
    }
}

impl<T, U, V> Drop for SoaSlice<T, U, V> {
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let (t_slice, u_slice, v_slice) = self.as_mut_slices();
        unsafe {
            ptr::drop_in_place(t_slice);
            ptr::drop_in_place(u_slice);
            ptr::drop_in_place(v_slice);
        }
    }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts<'slice, T, U, V>(
    data: *const u8,
    capacity: usize,
) -> &'slice SoaSlice<T, U, V> {
    unsafe { &*slice_from_raw_parts(data, capacity) }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts_mut<'slice, T, U, V>(
    data: *mut u8,
    capacity: usize,
) -> &'slice mut SoaSlice<T, U, V> {
    unsafe { &mut *slice_from_raw_parts_mut(data, capacity) }
}

#[inline]
pub(crate) unsafe fn from_len_in_bytes<'slice, T, U, V>(
    data: *const u8,
    len_in_bytes: usize,
) -> &'slice SoaSlice<T, U, V> {
    unsafe { &*slice_from_len_in_bytes(data, len_in_bytes) }
}

#[inline]
pub(crate) unsafe fn from_len_in_bytes_mut<'slice, T, U, V>(
    data: *mut u8,
    len_in_bytes: usize,
) -> &'slice mut SoaSlice<T, U, V> {
    unsafe { &mut *slice_from_len_in_bytes_mut(data, len_in_bytes) }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait SoaSliceIndex<T>: private_slice_index::Sealed
where
    T: ?Sized,
{
    type Ref<'a>
    where
        T: 'a;

    type RefMut<'a>
    where
        T: 'a;

    fn get(self, slice: &T) -> Option<Self::Ref<'_>>;

    fn get_mut(self, slice: &mut T) -> Option<Self::RefMut<'_>>;

    fn index(self, slice: &T) -> Self::Ref<'_>;

    fn index_mut(self, slice: &mut T) -> Self::RefMut<'_>;

    type ConstPtr;

    type MutPtr;

    unsafe fn get_unchecked(self, slice: *const T) -> Self::ConstPtr;

    unsafe fn get_unchecked_mut(self, slice: *mut T) -> Self::MutPtr;
}

unsafe impl<T, U, V> SoaSliceIndex<SoaSlice<T, U, V>> for usize {
    type Ref<'a> = (&'a T, &'a U, &'a V)
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (&'a mut T, &'a mut U, &'a mut V)
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        if self >= slice.len() {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked(slice);
            Some((&*t_ptr, &*u_ptr, &*v_ptr))
        }
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        if self >= slice.len() {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked_mut(slice);
            Some((&mut *t_ptr, &mut *u_ptr, &mut *v_ptr))
        }
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        match self.get(slice) {
            Some(value) => value,
            None => slice_index_usize_fail(slice.len(), self),
        }
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        let len = slice.len();
        match self.get_mut(slice) {
            Some(value) => value,
            None => slice_index_usize_fail(len, self),
        }
    }

    type ConstPtr = (*const T, *const U, *const V);

    type MutPtr = (*mut T, *mut U, *mut V);

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        unsafe {
            debug_assert!(
                self < (*slice).len(),
                "slice::get_unchecked requires that the index is within the slice",
            );
        }

        let buffer = slice as *const [u8];
        let ptr = buffer as _;
        let len = to_len::<T, U, V>(buffer.len());
        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr.add(self), u_ptr.add(self), v_ptr.add(self))
        }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        unsafe {
            debug_assert!(
                self < (*slice).len(),
                "slice::get_unchecked_mut requires that the index is within the slice",
            );
        }

        let buffer = slice as *mut [u8];
        let ptr = buffer as _;
        let len = to_len::<T, U, V>(buffer.len());
        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr.add(self), u_ptr.add(self), v_ptr.add(self))
        }
    }
}

unsafe impl<T, U, V> SoaSliceIndex<SoaSlice<T, U, V>> for ops::Range<usize> {
    type Ref<'a> = (&'a [T], &'a [U], &'a [V])
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (&'a mut [T], &'a mut [U], &'a mut [V])
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        if self.start > self.end || self.end > slice.len() {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked(slice);
            Some((&*t_ptr, &*u_ptr, &*v_ptr))
        }
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        if self.start > self.end || self.end > slice.len() {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked_mut(slice);
            Some((&mut *t_ptr, &mut *u_ptr, &mut *v_ptr))
        }
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        if self.start > self.end {
            slice_index_order_fail(self.start, self.end);
        } else if self.end > slice.len() {
            slice_end_index_len_fail(self.end, slice.len());
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked(slice);
            (&*t_ptr, &*u_ptr, &*v_ptr)
        }
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        if self.start > self.end {
            slice_index_order_fail(self.start, self.end);
        } else if self.end > slice.len() {
            slice_end_index_len_fail(self.end, slice.len());
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked_mut(slice);
            (&mut *t_ptr, &mut *u_ptr, &mut *v_ptr)
        }
    }

    type ConstPtr = (*const [T], *const [U], *const [V]);

    type MutPtr = (*mut [T], *mut [U], *mut [V]);

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        unsafe {
            debug_assert!(
                self.end >= self.start && self.end <= (*slice).len(),
                "slice::get_unchecked requires that the range is within the slice",
            );
        }

        let buffer = slice as *const [u8];
        let ptr = buffer as _;
        let len = to_len::<T, U, V>(buffer.len());
        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);

            let new_len = self.end.unchecked_sub(self.start);
            (
                ptr::slice_from_raw_parts(t_ptr.add(self.start), new_len),
                ptr::slice_from_raw_parts(u_ptr.add(self.start), new_len),
                ptr::slice_from_raw_parts(v_ptr.add(self.start), new_len),
            )
        }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        unsafe {
            debug_assert!(
                self.end >= self.start && self.end <= (*slice).len(),
                "slice::get_unchecked_mut requires that the range is within the slice",
            );
        }

        let buffer = slice as *const [u8];
        let ptr = buffer as _;
        let len = to_len::<T, U, V>(buffer.len());
        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);

            let new_len = self.end.unchecked_sub(self.start);
            (
                ptr::slice_from_raw_parts_mut(t_ptr.add(self.start), new_len),
                ptr::slice_from_raw_parts_mut(u_ptr.add(self.start), new_len),
                ptr::slice_from_raw_parts_mut(v_ptr.add(self.start), new_len),
            )
        }
    }
}

unsafe impl<T, U, V> SoaSliceIndex<SoaSlice<T, U, V>> for ops::RangeTo<usize> {
    type Ref<'a> = (&'a [T], &'a [U], &'a [V])
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (&'a mut [T], &'a mut [U], &'a mut [V])
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        (0..self.end).get(slice)
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        (0..self.end).get_mut(slice)
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        (0..self.end).index(slice)
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        (0..self.end).index_mut(slice)
    }

    type ConstPtr = (*const [T], *const [U], *const [V]);

    type MutPtr = (*mut [T], *mut [U], *mut [V]);

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        unsafe { (0..self.end).get_unchecked(slice) }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        unsafe { (0..self.end).get_unchecked_mut(slice) }
    }
}

unsafe impl<T, U, V> SoaSliceIndex<SoaSlice<T, U, V>> for ops::RangeFrom<usize> {
    type Ref<'a> = (&'a [T], &'a [U], &'a [V])
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (&'a mut [T], &'a mut [U], &'a mut [V])
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        (self.start..slice.len()).get(slice)
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        (self.start..slice.len()).get_mut(slice)
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        if self.start > slice.len() {
            slice_start_index_len_fail(self.start, slice.len());
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked(slice);
            (&*t_ptr, &*u_ptr, &*v_ptr)
        }
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        if self.start > slice.len() {
            slice_start_index_len_fail(self.start, slice.len());
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked_mut(slice);
            (&mut *t_ptr, &mut *u_ptr, &mut *v_ptr)
        }
    }

    type ConstPtr = (*const [T], *const [U], *const [V]);

    type MutPtr = (*mut [T], *mut [U], *mut [V]);

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        let buffer = slice as *const [u8];
        let len = to_len::<T, U, V>(buffer.len());
        unsafe { (self.start..len).get_unchecked(slice) }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        let buffer = slice as *const [u8];
        let len = to_len::<T, U, V>(buffer.len());
        unsafe { (self.start..len).get_unchecked_mut(slice) }
    }
}

unsafe impl<T, U, V> SoaSliceIndex<SoaSlice<T, U, V>> for ops::RangeFull {
    type Ref<'a> = (&'a [T], &'a [U], &'a [V])
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (&'a mut [T], &'a mut [U], &'a mut [V])
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        Some(slice.as_slices())
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        Some(slice.as_mut_slices())
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        slice.as_slices()
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        slice.as_mut_slices()
    }

    type ConstPtr = (*const [T], *const [U], *const [V]);

    type MutPtr = (*mut [T], *mut [U], *mut [V]);

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        let (t_slice, u_slice, v_slice) = unsafe { (*slice).as_slices() };
        (t_slice, u_slice, v_slice)
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        let (t_slice, u_slice, v_slice) = unsafe { (*slice).as_mut_slices() };
        (t_slice, u_slice, v_slice)
    }
}

/// Based on implementation of 2 methods:
/// - [`core::ops::RangeInclusive::into_slice_range()`]
/// - [`core::ops::RangeInclusive::is_empty()`] which replaces access to [`core::ops::RangeInclusive::exhausted`] private field
fn range_into_slice_range(range: ops::RangeInclusive<usize>) -> ops::Range<usize> {
    let exclusive_end = range.end() + 1;

    let exhausted = range.is_empty();
    let start = if exhausted {
        exclusive_end
    } else {
        *range.start()
    };

    start..exclusive_end
}

unsafe impl<T, U, V> SoaSliceIndex<SoaSlice<T, U, V>> for ops::RangeInclusive<usize> {
    type Ref<'a> = (&'a [T], &'a [U], &'a [V])
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (&'a mut [T], &'a mut [U], &'a mut [V])
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        if *self.end() == usize::MAX {
            return None;
        }
        range_into_slice_range(self).get(slice)
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        if *self.end() == usize::MAX {
            return None;
        }
        range_into_slice_range(self).get_mut(slice)
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        range_into_slice_range(self).index(slice)
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        range_into_slice_range(self).index_mut(slice)
    }

    type ConstPtr = (*const [T], *const [U], *const [V]);

    type MutPtr = (*mut [T], *mut [U], *mut [V]);

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        unsafe { range_into_slice_range(self).get_unchecked(slice) }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        unsafe { range_into_slice_range(self).get_unchecked_mut(slice) }
    }
}

unsafe impl<T, U, V> SoaSliceIndex<SoaSlice<T, U, V>> for ops::RangeToInclusive<usize> {
    type Ref<'a> = (&'a [T], &'a [U], &'a [V])
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (&'a mut [T], &'a mut [U], &'a mut [V])
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        (0..=self.end).get(slice)
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        (0..=self.end).get_mut(slice)
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        (0..=self.end).index(slice)
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        (0..=self.end).index_mut(slice)
    }

    type ConstPtr = (*const [T], *const [U], *const [V]);

    type MutPtr = (*mut [T], *mut [U], *mut [V]);

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        unsafe { (0..=self.end).get_unchecked(slice) }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        unsafe { (0..=self.end).get_unchecked_mut(slice) }
    }
}

/// Copy of private [`core::slice::index::into_range_unchecked()`] function.
fn into_range_unchecked(
    len: usize,
    (start, end): (ops::Bound<usize>, ops::Bound<usize>),
) -> ops::Range<usize> {
    use ops::Bound;
    let start = match start {
        Bound::Included(i) => i,
        Bound::Excluded(i) => i + 1,
        Bound::Unbounded => 0,
    };
    let end = match end {
        Bound::Included(i) => i + 1,
        Bound::Excluded(i) => i,
        Bound::Unbounded => len,
    };
    start..end
}

/// Copy of private [`core::slice::index::into_range()`] function.
fn into_range(
    len: usize,
    (start, end): (ops::Bound<usize>, ops::Bound<usize>),
) -> Option<ops::Range<usize>> {
    use ops::Bound;
    let start = match start {
        Bound::Included(start) => start,
        Bound::Excluded(start) => start.checked_add(1)?,
        Bound::Unbounded => 0,
    };

    let end = match end {
        Bound::Included(end) => end.checked_add(1)?,
        Bound::Excluded(end) => end,
        Bound::Unbounded => len,
    };

    // Don't bother with checking `start < end` and `end <= len`
    // since these checks are handled by `Range` impls

    Some(start..end)
}

/// Copy of private [`core::slice::index::into_slice_range()`] function.
fn into_slice_range(
    len: usize,
    (start, end): (ops::Bound<usize>, ops::Bound<usize>),
) -> ops::Range<usize> {
    use ops::Bound;
    let start = match start {
        Bound::Included(start) => start,
        Bound::Excluded(start) => start
            .checked_add(1)
            .unwrap_or_else(|| slice_start_index_overflow_fail()),
        Bound::Unbounded => 0,
    };

    let end = match end {
        Bound::Included(end) => end
            .checked_add(1)
            .unwrap_or_else(|| slice_end_index_overflow_fail()),
        Bound::Excluded(end) => end,
        Bound::Unbounded => len,
    };

    // Don't bother with checking `start < end` and `end <= len`
    // since these checks are handled by `Range` impls

    start..end
}

unsafe impl<T, U, V> SoaSliceIndex<SoaSlice<T, U, V>> for (ops::Bound<usize>, ops::Bound<usize>) {
    type Ref<'a> = (&'a [T], &'a [U], &'a [V])
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (&'a mut [T], &'a mut [U], &'a mut [V])
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        into_range(slice.len(), self)?.get(slice)
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        into_range(slice.len(), self)?.get_mut(slice)
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        into_slice_range(slice.len(), self).index(slice)
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        into_slice_range(slice.len(), self).index_mut(slice)
    }

    type ConstPtr = (*const [T], *const [U], *const [V]);

    type MutPtr = (*mut [T], *mut [U], *mut [V]);

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        let buffer = slice as *const [u8];
        let len = to_len::<T, U, V>(buffer.len());
        unsafe { into_range_unchecked(len, self).get_unchecked(slice) }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        let buffer = slice as *const [u8];
        let len = to_len::<T, U, V>(buffer.len());
        unsafe { into_range_unchecked(len, self).get_unchecked_mut(slice) }
    }
}

#[cold]
#[inline(never)]
#[track_caller]
fn slice_index_usize_fail(len: usize, index: usize) -> ! {
    panic!("index out of bounds: the len is {len} but the index is {index}")
}

#[cold]
#[inline(never)]
#[track_caller]
fn slice_index_order_fail(index: usize, end: usize) -> ! {
    panic!("slice index starts at {index} but ends at {end}");
}

#[inline]
#[track_caller]
fn slice_start_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range start index {index} out of range for slice of length {len}");
}

#[cold]
#[inline(never)]
#[track_caller]
fn slice_end_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range end index {index} out of range for slice of length {len}");
}

#[cold]
#[inline(never)]
#[track_caller]
const fn slice_end_index_overflow_fail() -> ! {
    panic!("attempted to index slice up to maximum usize");
}

#[cold]
#[inline(never)]
#[track_caller]
const fn slice_start_index_overflow_fail() -> ! {
    panic!("attempted to index slice from after maximum usize");
}

mod private_slice_index {
    use core::ops;

    pub trait Sealed {}

    impl Sealed for usize {}

    impl Sealed for ops::Range<usize> {}

    impl Sealed for ops::RangeTo<usize> {}

    impl Sealed for ops::RangeFrom<usize> {}

    impl Sealed for ops::RangeFull {}

    impl Sealed for ops::RangeInclusive<usize> {}

    impl Sealed for ops::RangeToInclusive<usize> {}

    impl Sealed for (ops::Bound<usize>, ops::Bound<usize>) {}
}
