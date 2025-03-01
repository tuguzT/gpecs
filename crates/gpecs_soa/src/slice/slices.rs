use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem,
    ops::{Index, IndexMut},
};

use alloc::vec::Vec;

use crate::{
    ptr::is_zst,
    set_len_on_drop::SetLenOnDrop,
    traits::{Soa, SoaToOwned},
    vec::SoaVec,
};

use super::{slice_index_usize_fail, IndexHelper, IndexHelperMut, Iter, IterMut, SoaSliceIndex};

pub struct SoaSlices<'a, T>
where
    T: Soa + 'a,
{
    context: &'a T::Context,
    ptrs: T::Ptrs,
    len: usize,
}

impl<'a, T> SoaSlices<'a, T>
where
    T: Soa,
{
    #[inline]
    pub fn new(context: &'a T::Context, slices: T::Slices<'a>) -> Self {
        let slices = T::slice_refs_as_slice_ptrs(context, slices);
        Self {
            context,
            ptrs: T::slice_ptrs_as_ptrs(context, slices),
            len: T::slice_ptrs_len(context, slices),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_> {
        let Self { context, ptrs, len } = *self;

        let slices = T::slices_from_raw_parts(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&T::Context, T::Slices<'_>) {
        let Self { context, ptrs, len } = *self;

        let slices = T::slices_from_raw_parts(context, ptrs, len);
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn into_slices(self) -> T::Slices<'a> {
        let Self { context, ptrs, len } = self;

        let slices = T::slices_from_raw_parts(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'a T::Context, T::Slices<'a>) {
        let Self { context, ptrs, len } = self;

        let slices = T::slices_from_raw_parts(context, ptrs, len);
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_ptrs(&self) -> T::Ptrs {
        let Self { ptrs, .. } = *self;
        ptrs
    }

    #[inline]
    pub fn into_parts(self) -> (&'a T::Context, T::Ptrs, usize) {
        let Self { context, ptrs, len } = self;
        (context, ptrs, len)
    }

    #[inline]
    pub unsafe fn from_parts(context: &'a T::Context, ptrs: T::Ptrs, len: usize) -> Self {
        Self { context, ptrs, len }
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Refs<'_>>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        index.get(context, slices)
    }

    #[inline]
    pub fn into_get<I>(self, index: I) -> Option<I::Refs<'a>>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        index.get(context, slices)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        let slices = T::slice_refs_as_slice_ptrs(context, slices);
        unsafe { index.get_unchecked(context, slices) }
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&self, index: I) -> I::Refs<'_>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        index.index(context, slices)
    }

    #[inline]
    #[track_caller]
    pub fn into_index<I>(self, index: I) -> I::Refs<'a>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        index.index(context, slices)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(*self)
    }

    #[inline]
    pub fn contains(&self, value: &T) -> bool
    where
        T::Refs<'a>: PartialEq<T>,
    {
        let mut iter = (*self).into_iter();
        iter.any(|item| item == *value)
    }

    #[inline]
    pub fn contains_by_refs<'r>(&self, refs: T::Refs<'r>) -> bool
    where
        T::Refs<'a>: PartialEq<T::Refs<'r>>,
    {
        let mut iter = (*self).into_iter();
        iter.any(|item| item == refs)
    }

    #[inline]
    pub fn to_vec(&self) -> SoaVec<T>
    where
        T::Refs<'a>: SoaToOwned<'a, Owned = T>,
        T::Context: Clone,
    {
        let len = self.len();
        let context = self.context().clone();
        let mut vec = SoaVec::with_context_and_capacity(context, len);

        let mut set_len_on_drop = SetLenOnDrop {
            vec: &mut vec,
            local_len: 0,
        };
        let ptrs = set_len_on_drop.vec.as_mut_ptrs();
        let context = set_len_on_drop.vec.context();
        for (index, refs) in (*self).into_iter().enumerate() {
            set_len_on_drop.local_len = index;
            unsafe {
                let dst = T::ptrs_add_mut(context, ptrs, index);
                refs.clone_into_ptrs(context, dst);
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
}

impl<T> Debug for SoaSlices<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlices").field(&slices).finish()
    }
}

impl<'a, T> AsRef<SoaSlices<'a, T>> for SoaSlices<'a, T>
where
    T: Soa,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlices<'_, T>
where
    for<'any> T: Soa<Slices<'any> = &'any [U]> + 'any,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> PartialEq for SoaSlices<'_, T>
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

impl<T> Eq for SoaSlices<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Eq,
{
}

impl<T> PartialOrd for SoaSlices<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Ord for SoaSlices<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Hash for SoaSlices<'_, T>
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

impl<T> Clone for SoaSlices<'_, T>
where
    T: Soa,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SoaSlices<'_, T> where T: Soa {}

impl<T, U, I> Index<I> for SoaSlices<'_, T>
where
    T: Soa,
    U: ?Sized,
    for<'a> I: IndexHelper<'a, T, Output = U>,
{
    type Output = U;

    fn index(&self, index: I) -> &Self::Output {
        SoaSlices::index(self, index)
    }
}

impl<'a, T> IntoIterator for &'a SoaSlices<'_, T>
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

impl<'a, T> IntoIterator for SoaSlices<'a, T>
where
    T: Soa,
{
    type Item = T::Refs<'a>;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

pub struct SoaSlicesMut<'a, T>
where
    T: Soa + 'a,
{
    context: &'a T::Context,
    ptrs: T::MutPtrs,
    len: usize,
}

impl<'a, T> SoaSlicesMut<'a, T>
where
    T: Soa,
{
    #[inline]
    pub fn new(context: &'a T::Context, slices: T::SlicesMut<'a>) -> Self {
        let slices = T::mut_slice_refs_as_slice_ptrs(context, slices);
        Self {
            context,
            ptrs: T::mut_slice_ptrs_as_ptrs(context, slices),
            len: T::slice_ptrs_len_mut(context, slices),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_> {
        let Self { context, ptrs, len } = *self;

        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        let slices = T::slice_ptrs_cast_const(context, slices);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&T::Context, T::Slices<'_>) {
        let Self { context, ptrs, len } = *self;

        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        let slices = T::slice_ptrs_cast_const(context, slices);
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&mut self) -> (&T::Context, T::SlicesMut<'_>) {
        let Self { context, ptrs, len } = *self;

        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        let slices = unsafe { T::slice_ptrs_to_slices_mut(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn into_slices(self) -> T::SlicesMut<'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'a T::Context, T::SlicesMut<'a>) {
        let Self { context, ptrs, len } = self;

        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        let slices = unsafe { T::slice_ptrs_to_slices_mut(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_ptrs(&self) -> T::Ptrs {
        let Self { context, ptrs, .. } = *self;
        T::ptrs_cast_const(context, ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> T::MutPtrs {
        let Self { ptrs, .. } = *self;
        ptrs
    }

    #[inline]
    pub fn into_parts(self) -> (&'a T::Context, T::MutPtrs, usize) {
        let Self { context, ptrs, len } = self;
        (context, ptrs, len)
    }

    #[inline]
    pub unsafe fn from_parts(context: &'a T::Context, ptrs: T::MutPtrs, len: usize) -> Self {
        Self { context, ptrs, len }
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Refs<'_>>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        index.get(context, slices)
    }

    #[inline]
    pub fn into_get<I>(self, index: I) -> Option<I::Refs<'a>>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = T::mut_slices_as_slices(context, slices);
        index.get(context, slices)
    }

    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::RefsMut<'_>>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        index.get_mut(context, slices)
    }

    #[inline]
    pub fn into_get_mut<I>(self, index: I) -> Option<I::RefsMut<'a>>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        index.get_mut(context, slices)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        let slices = T::slice_refs_as_slice_ptrs(context, slices);
        unsafe { index.get_unchecked(context, slices) }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        let slices = T::mut_slice_refs_as_slice_ptrs(context, slices);
        unsafe { index.get_unchecked_mut(context, slices) }
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&self, index: I) -> I::Refs<'_>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        index.index(context, slices)
    }

    #[inline]
    #[track_caller]
    pub fn into_index<I>(self, index: I) -> I::Refs<'a>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = T::mut_slices_as_slices(context, slices);
        index.index(context, slices)
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&mut self, index: I) -> I::RefsMut<'_>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        index.index_mut(context, slices)
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut<I>(self, index: I) -> I::RefsMut<'a>
    where
        I: SoaSliceIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        index.index_mut(context, slices)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        let Self { context, ptrs, len } = *self;
        let slices = unsafe {
            let ptrs = T::ptrs_cast_const(context, ptrs);
            SoaSlices::from_parts(context, ptrs, len)
        };
        Iter::new(slices)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let Self { context, ptrs, len } = *self;
        let slices = unsafe { Self::from_parts(context, ptrs, len) };
        IterMut::new(slices)
    }

    #[inline]
    pub fn contains(&self, value: &T) -> bool
    where
        T::Refs<'a>: PartialEq<T>,
    {
        let Self { context, ptrs, len } = *self;
        let slices = unsafe {
            let ptrs = T::ptrs_cast_const(context, ptrs);
            SoaSlices::from_parts(context, ptrs, len)
        };
        slices.contains(value)
    }

    #[inline]
    pub fn contains_by_refs<'r>(&self, refs: T::Refs<'r>) -> bool
    where
        T::Refs<'a>: PartialEq<T::Refs<'r>>,
    {
        let Self { context, ptrs, len } = *self;
        let slices = unsafe {
            let ptrs = T::ptrs_cast_const(context, ptrs);
            SoaSlices::<T>::from_parts(context, ptrs, len)
        };
        slices.contains_by_refs(refs)
    }

    #[inline]
    pub fn to_vec(&self) -> SoaVec<T>
    where
        T::Refs<'a>: SoaToOwned<'a, Owned = T>,
        T::Context: Clone,
    {
        let len = self.len();
        let context = self.context().clone();
        let mut vec = SoaVec::with_context_and_capacity(context, len);

        let mut set_len_on_drop = SetLenOnDrop {
            vec: &mut vec,
            local_len: 0,
        };
        let ptrs = set_len_on_drop.vec.as_mut_ptrs();
        let context = set_len_on_drop.vec.context();

        let slices = {
            let Self { context, ptrs, len } = *self;
            unsafe { SoaSlices::<T>::from_parts(context, T::ptrs_cast_const(context, ptrs), len) }
        };
        for (index, refs) in slices.into_iter().enumerate() {
            set_len_on_drop.local_len = index;
            unsafe {
                let dst = T::ptrs_add_mut(context, ptrs, index);
                refs.clone_into_ptrs(context, dst);
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
    pub fn clone_from_slices<'src>(&mut self, src: SoaSlices<'src, T>)
    where
        T::Refs<'src>: SoaToOwned<'src, Owned = T>,
    {
        let len = self.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        for index in 0..len {
            unsafe {
                let dst = self.get_unchecked_mut(index);
                let context = self.context();
                let src = T::ptrs_to_refs(context, src.get_unchecked(index));
                T::ptrs_drop_in_place(context, dst);
                src.clone_into_ptrs(context, dst);
            }
        }
    }

    #[inline]
    #[track_caller]
    pub fn copy_from_slices(&mut self, src: SoaSlices<'_, T>)
    where
        T: Copy,
    {
        let len = self.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
        // checked to have the same length. The slices cannot overlap because
        // mutable references are exclusive.
        unsafe {
            let dst = self.as_mut_ptrs();
            let context = self.context();
            T::ptrs_copy_nonoverlapping(context, src.as_ptrs(), dst, len);
        }
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

        // call `get_unchecked_mut` directly on slice pointers to avoid creating multiple mutable references
        let (context, slices) = self.as_mut_slices_with_context();
        let slices = T::mut_slice_refs_as_slice_ptrs(context, slices);
        unsafe {
            let a = SoaSliceIndex::<T>::get_unchecked_mut(a, context, slices);
            let b = SoaSliceIndex::<T>::get_unchecked_mut(b, context, slices);
            T::ptrs_swap(context, a, b)
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
        self.sort_impl(|context, indices| {
            indices.sort_by(|&a, &b| {
                let a = unsafe {
                    let ptrs = T::ptrs_add_mut(context, ptrs, a);
                    let ptrs = T::ptrs_cast_const(context, ptrs);
                    T::ptrs_to_refs(context, ptrs)
                };
                let b = unsafe {
                    let ptrs = T::ptrs_add_mut(context, ptrs, b);
                    let ptrs = T::ptrs_cast_const(context, ptrs);
                    T::ptrs_to_refs(context, ptrs)
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
        self.sort_impl(|context, indices| {
            indices.sort_by_key(|&index| unsafe {
                let ptrs = T::ptrs_add_mut(context, ptrs, index);
                let ptrs = T::ptrs_cast_const(context, ptrs);
                let refs = T::ptrs_to_refs(context, ptrs);
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
        self.sort_impl(|context, indices| {
            indices.sort_by_cached_key(|&index| unsafe {
                let ptrs = T::ptrs_add_mut(context, ptrs, index);
                let ptrs = T::ptrs_cast_const(context, ptrs);
                let refs = T::ptrs_to_refs(context, ptrs);
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
        self.sort_impl(|context, indices| {
            indices.sort_unstable_by(|&a, &b| {
                let a = unsafe {
                    let ptrs = T::ptrs_add_mut(context, ptrs, a);
                    let ptrs = T::ptrs_cast_const(context, ptrs);
                    T::ptrs_to_refs(context, ptrs)
                };
                let b = unsafe {
                    let ptrs = T::ptrs_add_mut(context, ptrs, b);
                    let ptrs = T::ptrs_cast_const(context, ptrs);
                    T::ptrs_to_refs(context, ptrs)
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
        self.sort_impl(|context, indices| {
            indices.sort_unstable_by_key(|&index| unsafe {
                let ptrs = T::ptrs_add_mut(context, ptrs, index);
                let ptrs = T::ptrs_cast_const(context, ptrs);
                let refs = T::ptrs_to_refs(context, ptrs);
                f(refs)
            })
        })
    }

    fn sort_impl<F>(&mut self, f: F)
    where
        F: FnOnce(&T::Context, &mut [usize]),
    {
        let len = self.len();
        if is_zst::<T>() || len < 2 {
            return;
        }

        let mut permutation: Vec<_> = (0..len).collect();
        f(self.context(), &mut permutation);

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

#[inline(never)]
#[cold]
#[track_caller]
fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
    panic!(
        "source slice length ({}) does not match destination slice length ({})",
        src_len, dst_len,
    );
}

impl<'a, T> From<SoaSlicesMut<'a, T>> for SoaSlices<'a, T>
where
    T: Soa,
{
    fn from(slices: SoaSlicesMut<'a, T>) -> Self {
        let SoaSlicesMut { context, ptrs, len } = slices;
        unsafe { Self::from_parts(context, T::ptrs_cast_const(context, ptrs), len) }
    }
}

impl<T> Debug for SoaSlicesMut<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlices").field(&slices).finish()
    }
}

impl<'a, T> AsRef<SoaSlicesMut<'a, T>> for SoaSlicesMut<'a, T>
where
    T: Soa,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlicesMut<'_, T>
where
    for<'any> T: Soa<Slices<'any> = &'any [U]> + 'any,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<'a, T> AsMut<SoaSlicesMut<'a, T>> for SoaSlicesMut<'a, T>
where
    T: Soa,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U> AsMut<[U]> for SoaSlicesMut<'_, T>
where
    for<'any> T: Soa<SlicesMut<'any> = &'any mut [U]> + 'any,
{
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices()
    }
}

impl<T> PartialEq for SoaSlicesMut<'_, T>
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

impl<T> Eq for SoaSlicesMut<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Eq,
{
}

impl<T> PartialOrd for SoaSlicesMut<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Ord for SoaSlicesMut<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Hash for SoaSlicesMut<'_, T>
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

impl<T, U, I> Index<I> for SoaSlicesMut<'_, T>
where
    T: Soa,
    U: ?Sized,
    for<'a> I: IndexHelper<'a, T, Output = U>,
{
    type Output = U;

    fn index(&self, index: I) -> &Self::Output {
        SoaSlicesMut::index(self, index)
    }
}

impl<T, U, I> IndexMut<I> for SoaSlicesMut<'_, T>
where
    T: Soa,
    U: ?Sized,
    for<'a> I: IndexHelperMut<'a, T, Output = U>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        SoaSlicesMut::index_mut(self, index)
    }
}

impl<'a, T> IntoIterator for &'a SoaSlicesMut<'_, T>
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

impl<'a, T> IntoIterator for &'a mut SoaSlicesMut<'_, T>
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

impl<'a, T> IntoIterator for SoaSlicesMut<'a, T>
where
    T: Soa,
{
    type Item = T::RefsMut<'a>;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterMut::new(self)
    }
}
