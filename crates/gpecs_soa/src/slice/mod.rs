use alloc::{boxed::Box, vec::Vec};
use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr::{self, NonNull},
    slice,
};

use crate::{
    ptr::{
        min_size_of, ptrs, slice_from_len_in_bytes, slice_from_len_in_bytes_mut,
        slice_from_raw_parts, slice_from_raw_parts_mut, to_len, BufferAlign,
    },
    vec::SoaVec,
};

pub use self::{
    index::SoaSliceIndex,
    iter::{Iter, IterMut},
};

mod index;
mod iter;

#[repr(C)]
pub struct SoaSlice<T, U, V> {
    align: BufferAlign<T, U, V>,
    buffer: [u8],
}

impl<T, U, V> SoaSlice<T, U, V> {
    #[inline]
    pub const fn len(&self) -> usize {
        match self.capacity_in_bytes() {
            0 => 0,
            _ => unsafe { ptr::read(self.as_ptr().cast()) },
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn capacity_in_bytes(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        if min_size_of::<T, U, V>() == 0 {
            usize::MAX
        } else {
            let len_in_bytes = self.capacity_in_bytes();
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
    pub fn into_vec(self: Box<Self>) -> SoaVec<T, U, V> {
        let length = self.len();
        let capacity_in_bytes = self.capacity_in_bytes();
        let ptr = Box::into_raw(self).cast();
        unsafe { SoaVec::from_capacity_in_bytes(ptr, length, capacity_in_bytes) }
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

    #[inline]
    pub fn iter(&self) -> Iter<'_, T, U, V> {
        Iter::new(self)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T, U, V> {
        IterMut::new(self)
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
        f.debug_struct("SoaSlice")
            .field("t_slice", &t_slice)
            .field("u_slice", &u_slice)
            .field("v_slice", &v_slice)
            .finish()
    }
}

impl<T, U, V> Default for &SoaSlice<T, U, V> {
    fn default() -> Self {
        let data = NonNull::<BufferAlign<T, U, V>>::dangling().as_ptr().cast();
        unsafe { from_len_in_bytes(data, 0) }
    }
}

impl<T, U, V> Default for &mut SoaSlice<T, U, V> {
    fn default() -> Self {
        let data = NonNull::<BufferAlign<T, U, V>>::dangling().as_ptr().cast();
        unsafe { from_len_in_bytes_mut(data, 0) }
    }
}

impl<T, U, V> Default for Box<SoaSlice<T, U, V>> {
    fn default() -> Self {
        let data = NonNull::<BufferAlign<T, U, V>>::dangling().as_ptr().cast();
        unsafe { Box::from_raw(slice_from_len_in_bytes_mut(data, 0)) }
    }
}

impl<T, U, V> AsRef<SoaSlice<T, U, V>> for SoaSlice<T, U, V> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U, V> AsMut<SoaSlice<T, U, V>> for SoaSlice<T, U, V> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U, V> Hash for SoaSlice<T, U, V>
where
    T: Hash,
    U: Hash,
    V: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let len = self.len();
        state.write_usize(len);

        let (t_slice, u_slice, v_slice) = self.as_slices();
        t_slice.hash(state);
        u_slice.hash(state);
        v_slice.hash(state);
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

impl<'a, T, U, V> IntoIterator for &'a SoaSlice<T, U, V> {
    type Item = (&'a T, &'a U, &'a V);
    type IntoIter = Iter<'a, T, U, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, U, V> IntoIterator for &'a mut SoaSlice<T, U, V> {
    type Item = (&'a mut T, &'a mut U, &'a mut V);
    type IntoIter = IterMut<'a, T, U, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
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
