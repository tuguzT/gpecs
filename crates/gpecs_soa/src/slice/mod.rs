use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};
use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem, ops,
    ptr::{self, NonNull},
};
use index::slice_index_usize_fail;

use crate::{
    ptr::{is_zst, ptrs, slice_from_raw_parts, slice_from_raw_parts_mut, BufferData, SoaSlicePtr},
    set_len_on_drop::SetLenOnDrop,
    traits::{Soa, SoaToOwned},
    vec::{IntoIter, SoaVec},
};

use self::index::{
    slice_end_index_len_fail, slice_end_index_overflow_fail, slice_index_order_fail,
    slice_start_index_overflow_fail,
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
    pub fn as_ptr(&self) -> *const BufferData<T> {
        self.buffer.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut BufferData<T> {
        self.buffer.as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> T::Ptrs {
        let ptr = self.as_ptr().cast_mut();
        let len = self.capacity();

        unsafe {
            let ptrs = ptrs::<T>(ptr, len).unwrap_unchecked();
            T::ptrs_cast_const(ptrs)
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> T::MutPtrs {
        let ptr = self.as_mut_ptr();
        let len = self.capacity();

        unsafe { ptrs::<T>(ptr, len).unwrap_unchecked() }
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
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(self)
    }

    #[inline]
    pub fn contains<'me>(&'me self, value: &T) -> bool
    where
        T::Refs<'me>: PartialEq<T>,
    {
        self.iter().any(|element| element == *value)
    }

    #[inline]
    pub fn contains_by_refs<'me, 'r>(&'me self, refs: T::Refs<'r>) -> bool
    where
        T::Refs<'me>: PartialEq<T::Refs<'r>>,
    {
        self.iter().any(|element| element == refs)
    }

    #[inline]
    pub fn into_vec(self: Box<Self>) -> SoaVec<T> {
        let length = self.len();
        let capacity = self.capacity();
        let ptr = Box::into_raw(self).cast();
        unsafe { SoaVec::from_raw_parts(ptr, length, capacity) }
    }

    #[inline]
    pub fn to_vec<'me>(&'me self) -> SoaVec<T>
    where
        T::Refs<'me>: SoaToOwned<'me, Owned = T>,
    {
        let len = self.len();
        let mut vec = SoaVec::with_capacity(len);

        let mut set_len_on_drop = SetLenOnDrop {
            vec: &mut vec,
            local_len: 0,
        };
        let ptrs = set_len_on_drop.vec.as_mut_ptrs();
        for (index, refs) in self.iter().enumerate() {
            set_len_on_drop.local_len = index;
            unsafe {
                let dst = T::ptrs_add_mut(ptrs, index);
                refs.clone_into_ptrs(dst);
            }
        }
        mem::forget(set_len_on_drop);

        // SAFETY:
        // the vec was allocated and initialized above to at least this length.
        unsafe {
            vec.set_len(len);
        }
        vec
    }

    #[inline]
    #[track_caller]
    pub fn clone_from_slice<'src>(&mut self, src: &'src SoaSlice<T>)
    where
        T::Refs<'src>: SoaToOwned<'src, Owned = T>,
    {
        // The panic code path was put into a cold function to not bloat the
        // call site.
        #[inline(never)]
        #[cold]
        #[track_caller]
        fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
            panic!(
                "source slice length ({}) does not match destination slice length ({})",
                src_len, dst_len,
            );
        }

        let len = self.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        for index in 0..len {
            unsafe {
                let dst = self.get_unchecked_mut(index);
                let src = T::as_refs(src.get_unchecked(index));
                src.clone_into_ptrs(dst);
            }
        }
    }

    #[inline]
    #[track_caller]
    pub fn copy_from_slice(&mut self, src: &SoaSlice<T>)
    where
        T: Copy,
    {
        // The panic code path was put into a cold function to not bloat the
        // call site.
        #[inline(never)]
        #[cold]
        #[track_caller]
        fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
            panic!(
                "source slice length ({}) does not match destination slice length ({})",
                src_len, dst_len,
            );
        }

        let len = self.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
        // checked to have the same length. The slices cannot overlap because
        // mutable references are exclusive.
        unsafe {
            T::ptrs_copy_nonoverlapping(src.as_ptrs(), self.as_mut_ptrs(), len);
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
    #[track_caller]
    pub fn index<I>(&self, index: I) -> I::Ref<'_>
    where
        I: SoaSliceIndex<Self>,
    {
        index.index(self)
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&mut self, index: I) -> I::RefMut<'_>
    where
        I: SoaSliceIndex<Self>,
    {
        index.index_mut(self)
    }

    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: usize, b: usize) {
        let len = self.len();
        if a >= len {
            slice_index_usize_fail(len, a);
        }
        if b >= len {
            slice_index_usize_fail(len, b);
        }

        // call `get_unchecked_mut` directly on pointer to avoid creating multiple mutable references
        let slice = ptr::from_mut(self);
        unsafe {
            let a = a.get_unchecked_mut(slice);
            let b = b.get_unchecked_mut(slice);
            T::ptrs_swap(a, b)
        }
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
    #[inline]
    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        self.as_slices() != other.as_slices()
    }
}

impl<T> Eq for SoaSlice<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Eq,
{
}

impl<T> PartialOrd for SoaSlice<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Ord for SoaSlice<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(&self.as_slices(), &other.as_slices())
    }
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
        let slices = T::mut_slice_refs_as_slice_ptrs(slices);
        unsafe { T::slices_drop_in_place(slices) }
    }
}

impl<T> ToOwned for SoaSlice<T>
where
    T: Soa,
    for<'any> T::Refs<'any>: SoaToOwned<'any, Owned = T> + 'any,
{
    type Owned = SoaVec<T>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.to_vec()
    }

    #[inline]
    fn clone_into(&self, target: &mut Self::Owned) {
        // decide if this impl will be better:
        // https://github.com/rust-lang/rust/blob/019fc4de2f3d49a2ef862d180599194d2be05193/library/alloc/src/slice.rs#L860

        target.clear();
        target.extend_from_slice(self);
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

/// Just a copy of unstable [`core::slice::range`]
#[track_caller]
#[must_use]
pub(crate) fn slice_range<R>(range: R, bounds: ops::RangeTo<usize>) -> ops::Range<usize>
where
    R: ops::RangeBounds<usize>,
{
    let len = bounds.end;

    let start = match range.start_bound() {
        ops::Bound::Included(&start) => start,
        ops::Bound::Excluded(start) => start
            .checked_add(1)
            .unwrap_or_else(|| slice_start_index_overflow_fail()),
        ops::Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        ops::Bound::Included(end) => end
            .checked_add(1)
            .unwrap_or_else(|| slice_end_index_overflow_fail()),
        ops::Bound::Excluded(&end) => end,
        ops::Bound::Unbounded => len,
    };

    if start > end {
        slice_index_order_fail(start, end);
    }
    if end > len {
        slice_end_index_len_fail(end, len);
    }

    ops::Range { start, end }
}
