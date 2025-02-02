use alloc::{boxed::Box, vec::Vec};
use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr::{self, NonNull},
};

use crate::{
    ptr::{is_zst, ptrs, slice_from_raw_parts, slice_from_raw_parts_mut, BufferData, SoaSlicePtr},
    soa::Soa,
    vec::{IntoIter, SoaVec},
};

pub use self::{
    index::SoaSliceIndex,
    iter::{Iter, IterMut},
};

mod index;
mod iter;

#[repr(transparent)]
pub struct SoaSlice<T>
where
    T: Soa,
{
    buffer: [BufferData<T>],
}

impl<T> SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ptr::from_ref(self).len() }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity_in_bytes(&self) -> usize {
        ptr::from_ref(self).capacity_in_bytes()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        if is_zst::<T>() {
            return usize::MAX;
        }
        ptr::from_ref(self).capacity()
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const BufferData<T> {
        self.buffer.as_ptr()
    }

    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut BufferData<T> {
        self.buffer.as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> T::Ptrs {
        let ptr = self.as_ptr().cast_mut();
        let len = self.capacity();

        unsafe {
            let ptrs = ptrs::<T>(ptr, len);
            T::ptrs_cast_const(ptrs)
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> T::MutPtrs {
        let ptr = self.as_mut_ptr();
        let len = self.capacity();

        unsafe { ptrs::<T>(ptr, len) }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_> {
        let ptrs = self.as_ptrs();
        let len = self.len();

        let slices = T::slices_from_raw_parts(ptrs, len);
        unsafe { T::slices_as_refs(slices) }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_> {
        let ptrs = self.as_mut_ptrs();
        let len = self.len();

        let slices = T::slices_from_raw_parts_mut(ptrs, len);
        unsafe { T::mut_slices_as_refs(slices) }
    }

    #[inline]
    pub fn into_vec(self: Box<Self>) -> SoaVec<T> {
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
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptr
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
        let ptrs_a = {
            let refs = self.index_mut(a);
            T::mut_refs_as_ptrs(refs)
        };
        let ptrs_b = {
            let refs = self.index_mut(b);
            T::mut_refs_as_ptrs(refs)
        };

        unsafe { T::ptrs_swap(ptrs_a, ptrs_b) }
    }

    #[inline]
    pub fn sort(&mut self)
    where
        for<'any> T::Refs<'any>: Ord,
    {
        self.sort_by(|a, b| Ord::cmp(&a, &b))
    }

    #[inline]
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        for<'any> F: FnMut(T::Refs<'any>, T::Refs<'any>) -> cmp::Ordering,
    {
        let ptrs = self.as_mut_ptrs();
        self.sort_impl(|indices| {
            indices.sort_by(|&a, &b| {
                let a = unsafe {
                    let ptrs = T::ptrs_add_mut(ptrs, a);
                    let ptrs = T::ptrs_cast_const(ptrs);
                    T::as_refs(ptrs)
                };
                let b = unsafe {
                    let ptrs = T::ptrs_add_mut(ptrs, b);
                    let ptrs = T::ptrs_cast_const(ptrs);
                    T::as_refs(ptrs)
                };
                compare(a, b)
            })
        })
    }

    #[inline]
    pub fn sort_by_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut(T::Refs<'_>) -> K,
        K: Ord,
    {
        let ptrs = self.as_mut_ptrs();
        self.sort_impl(|indices| {
            indices.sort_by_key(|&index| unsafe {
                let ptrs = T::ptrs_add_mut(ptrs, index);
                let ptrs = T::ptrs_cast_const(ptrs);
                let refs = T::as_refs(ptrs);
                f(refs)
            })
        })
    }

    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut(T::Refs<'_>) -> K,
        K: Ord,
    {
        let ptrs = self.as_mut_ptrs();
        self.sort_impl(|indices| {
            indices.sort_by_cached_key(|&index| unsafe {
                let ptrs = T::ptrs_add_mut(ptrs, index);
                let ptrs = T::ptrs_cast_const(ptrs);
                let refs = T::as_refs(ptrs);
                f(refs)
            })
        })
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'any> T::Refs<'any>: Ord,
    {
        self.sort_unstable_by(|a, b| Ord::cmp(&a, &b))
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, mut compare: F)
    where
        for<'any> F: FnMut(T::Refs<'any>, T::Refs<'any>) -> cmp::Ordering,
    {
        let ptrs = self.as_mut_ptrs();
        self.sort_impl(|indices| {
            indices.sort_unstable_by(|&a, &b| {
                let a = unsafe {
                    let ptrs = T::ptrs_add_mut(ptrs, a);
                    let ptrs = T::ptrs_cast_const(ptrs);
                    T::as_refs(ptrs)
                };
                let b = unsafe {
                    let ptrs = T::ptrs_add_mut(ptrs, b);
                    let ptrs = T::ptrs_cast_const(ptrs);
                    T::as_refs(ptrs)
                };
                compare(a, b)
            })
        })
    }

    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut(T::Refs<'_>) -> K,
        K: Ord,
    {
        let ptrs = self.as_mut_ptrs();
        self.sort_impl(|indices| {
            indices.sort_unstable_by_key(|&index| unsafe {
                let ptrs = T::ptrs_add_mut(ptrs, index);
                let ptrs = T::ptrs_cast_const(ptrs);
                let refs = T::as_refs(ptrs);
                f(refs)
            })
        })
    }

    fn sort_impl<F>(&mut self, f: F)
    where
        F: FnOnce(&mut [usize]),
    {
        let len = self.len();
        if is_zst::<T>() || len < 2 {
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
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(self)
    }
}

impl<T> Debug for SoaSlice<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlice").field(&slices).finish()
    }
}

impl<T> Default for &SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    fn default() -> Self {
        let data = NonNull::<BufferData<T>>::dangling().as_ptr().cast();
        unsafe { from_raw_parts(data, 0, 0) }
    }
}

impl<T> Default for &mut SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    fn default() -> Self {
        let data = NonNull::<BufferData<T>>::dangling().as_ptr().cast();
        unsafe { from_raw_parts_mut(data, 0, 0) }
    }
}

impl<T> Default for Box<SoaSlice<T>>
where
    T: Soa,
{
    #[inline]
    fn default() -> Self {
        let data = NonNull::<BufferData<T>>::dangling().as_ptr().cast();
        unsafe { Box::from_raw(slice_from_raw_parts_mut(data, 0, 0)) }
    }
}

impl<T> AsRef<SoaSlice<T>> for SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T> AsMut<SoaSlice<T>> for SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T> PartialEq for SoaSlice<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slices() == other.as_slices()
    }
}

impl<T> Eq for SoaSlice<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Eq,
{
}

impl<T> Hash for SoaSlice<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let len = self.len();
        state.write_usize(len);

        let slices = self.as_slices();
        slices.hash(state);
    }
}

impl<T> Drop for SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let slices = self.as_mut_slices();
        let slices = T::mut_slice_refs_as_ptrs(slices);
        unsafe { T::slices_drop_in_place(slices) }
    }
}

impl<'a, T> IntoIterator for &'a SoaSlice<T>
where
    T: Soa,
{
    type Item = T::Refs<'a>;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a Box<SoaSlice<T>>
where
    T: Soa,
{
    type Item = T::Refs<'a>;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SoaSlice<T>
where
    T: Soa,
{
    type Item = T::RefsMut<'a>;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a mut Box<SoaSlice<T>>
where
    T: Soa,
{
    type Item = T::RefsMut<'a>;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for Box<SoaSlice<T>>
where
    T: Soa,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts<'slice, T>(
    data: *const BufferData<T>,
    len: usize,
    capacity: usize,
) -> &'slice SoaSlice<T>
where
    T: Soa,
{
    unsafe { &*slice_from_raw_parts(data, len, capacity) }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts_mut<'slice, T>(
    data: *mut BufferData<T>,
    len: usize,
    capacity: usize,
) -> &'slice mut SoaSlice<T>
where
    T: Soa,
{
    unsafe { &mut *slice_from_raw_parts_mut(data, len, capacity) }
}
