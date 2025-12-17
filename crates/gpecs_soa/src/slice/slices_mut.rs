use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    layout::is_zst,
    slice::{
        IndexHelper, IndexHelperMut, Iter, IterMut, RawIter, RawIterMut, SoaSliceMutPtrs,
        SoaSlicePtrs, SoaSlicePtrsIndex, SoaSlices, SoaSlicesIndex, assert::slice_index_usize_fail,
    },
    traits::{
        MutPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs, Soa, SoaCloneToUninit,
    },
};

pub struct SoaSlicesMut<'c, 'a, T>
where
    T: RawSoa + ?Sized + 'a,
{
    ptrs: SoaSliceMutPtrs<'c, T>,
    phantom: PhantomData<&'a ()>,
}

impl<'c, T> SoaSlicesMut<'c, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { ptrs, .. } = self;
        ptrs.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.as_ptrs_with_context()
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, T> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&T::Context, MutPtrs<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.as_mut_ptrs_with_context()
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, T> {
        let (_, ptrs) = self.as_slice_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&T::Context, SlicePtrs<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.as_slice_ptrs_with_context()
    }

    #[inline]
    pub fn as_slice_mut_ptrs(&mut self) -> SliceMutPtrs<'_, T> {
        let (_, ptrs) = self.as_slice_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(&mut self) -> (&T::Context, SliceMutPtrs<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.as_slice_mut_ptrs_with_context()
    }

    #[inline]
    pub fn slice_ptrs(&self) -> SoaSlicePtrs<'_, T> {
        let Self { ptrs, .. } = self;
        ptrs.clone().cast_const()
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SoaSlicePtrs<'c, T> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn slice_mut_ptrs(&mut self) -> SoaSliceMutPtrs<'_, T> {
        let Self { ptrs, .. } = self;
        ptrs.clone()
    }

    #[inline]
    pub fn into_slice_mut_ptrs(self) -> SoaSliceMutPtrs<'c, T> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, '_, T> {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        let (context, ptrs) = ptrs.as_ptrs_with_context();
        unsafe { SoaSlices::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub fn slices_mut(&mut self) -> SoaSlicesMut<'_, '_, T> {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        let (context, ptrs) = ptrs.as_mut_ptrs_with_context();
        unsafe { SoaSlicesMut::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, MutPtrs<'c, T>, usize) {
        let Self { ptrs, .. } = self;
        ptrs.into_parts()
    }

    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, ptrs: MutPtrs<'c, T>, len: usize) -> Self {
        Self {
            ptrs: unsafe { SoaSliceMutPtrs::from_parts(context, ptrs, len) },
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs<'_>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context<I>(&self, index: I) -> (&T::Context, I::Ptrs<'_>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.get_unchecked_with_context(index) }
    }

    #[inline]
    pub unsafe fn into_get_unchecked<I>(self, index: I) -> I::Ptrs<'c>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, I::Ptrs<'c>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.into_get_unchecked_with_context(index) }
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs<'_>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_mut_with_context<I>(
        &mut self,
        index: I,
    ) -> (&T::Context, I::MutPtrs<'_>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.get_unchecked_mut_with_context(index) }
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut<I>(self, index: I) -> I::MutPtrs<'c>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, I::MutPtrs<'c>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.into_get_unchecked_mut_with_context(index) }
    }

    #[inline]
    pub fn raw_iter(&self) -> RawIter<'_, T> {
        let (_, iter) = self.raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_with_context(&self) -> (&T::Context, RawIter<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.iter_with_context()
    }

    #[inline]
    pub fn raw_iter_mut(&mut self) -> RawIterMut<'_, T> {
        let (_, iter) = self.raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_mut_with_context(&mut self) -> (&T::Context, RawIterMut<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.iter_mut_with_context()
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'c, T> {
        let (_, iter) = self.into_raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_with_context(self) -> (&'c T::Context, RawIter<'c, T>) {
        let (context, iter) = self.into_raw_iter_mut_with_context();
        (context, iter.cast_const())
    }

    #[inline]
    pub fn into_raw_iter_mut(self) -> RawIterMut<'c, T> {
        let (_, iter) = self.into_raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_mut_with_context(self) -> (&'c T::Context, RawIterMut<'c, T>) {
        let Self { ptrs, .. } = self;
        ptrs.into_iter_with_context()
    }

    #[inline]
    #[track_caller]
    pub fn copy_from_slices(&mut self, src: &SoaSlices<T>)
    where
        T::Fields: Copy,
    {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
        // checked to have the same length. The slices cannot overlap because
        // mutable references are exclusive.
        let (context, dst) = ptrs.as_mut_ptrs_with_context();
        unsafe { context.ptrs_copy_nonoverlapping(src.as_ptrs(), dst, len) }
    }

    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: usize, b: usize) {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        if a >= len {
            slice_index_usize_fail(len, a);
        }
        if b >= len {
            slice_index_usize_fail(len, b);
        }

        // call `get_unchecked_mut` directly on slice pointers to avoid creating multiple mutable references
        let (context, slices) = ptrs.as_slice_mut_ptrs_with_context();
        unsafe {
            let a = SoaSlicePtrsIndex::<T>::get_unchecked_mut(a, context, slices.clone());
            let b = SoaSlicePtrsIndex::<T>::get_unchecked_mut(b, context, slices);
            context.ptrs_swap(a, b);
        }
    }

    pub(crate) fn sort_impl<P, F>(&mut self, mut permutation: P, f: F)
    where
        P: AsMut<[usize]>,
        F: FnOnce(&mut Self, &mut [usize]),
    {
        #[inline(never)]
        #[cold]
        #[track_caller]
        fn permutation_len_fail(permutation_len: usize, len: usize) -> ! {
            panic!("permutation must be at least {len} long, but its length is {permutation_len}")
        }

        let len = self.len();
        let permutation = permutation.as_mut();
        if permutation.len() < len {
            permutation_len_fail(permutation.len(), len);
        }

        let context = self.context();
        if is_zst::<T>(context) || len < 2 {
            return;
        }

        f(self, permutation);

        // were taken from `sort_by_cached_key()` method of slice primitive
        for src in 0..len {
            let mut dst = permutation[src];
            while dst < src {
                dst = permutation[dst];
            }
            permutation[src] = dst;
            self.swap(src, dst);
        }
    }
}

impl<'c, 'a, T> SoaSlicesMut<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: T::SlicesMut<'c, 'a>) -> Self {
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        Self {
            ptrs: SoaSliceMutPtrs::new(context, slices),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&T::Context, T::Slices<'_, '_>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.as_slice_ptrs_with_context();
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_, '_> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&mut self) -> (&T::Context, T::SlicesMut<'_, '_>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.as_slice_mut_ptrs_with_context();
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn into_slices(self) -> T::SlicesMut<'c, 'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'c T::Context, T::SlicesMut<'c, 'a>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.into_slice_mut_ptrs_with_context();
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Refs<'_, '_>>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.get_with_context(index);
        refs
    }

    #[inline]
    pub fn get_with_context<I>(&self, index: I) -> (&T::Context, Option<I::Refs<'_, '_>>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.get(context, slices))
    }

    #[inline]
    pub fn into_get<I>(self, index: I) -> Option<I::Refs<'c, 'a>>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.into_get_with_context(index);
        refs
    }

    #[inline]
    pub fn into_get_with_context<I>(self, index: I) -> (&'c T::Context, Option<I::Refs<'c, 'a>>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = T::slices_mut_as_slices(context, slices);
        (context, index.get(context, slices))
    }

    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::RefsMut<'_, '_>>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.get_mut_with_context(index);
        refs
    }

    #[inline]
    pub fn get_mut_with_context<I>(&mut self, index: I) -> (&T::Context, Option<I::RefsMut<'_, '_>>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        (context, index.get_mut(context, slices))
    }

    #[inline]
    pub fn into_get_mut<I>(self, index: I) -> Option<I::RefsMut<'c, 'a>>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.into_get_mut_with_context(index);
        refs
    }

    #[inline]
    pub fn into_get_mut_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, Option<I::RefsMut<'c, 'a>>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.get_mut(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&self, index: I) -> I::Refs<'_, '_>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context<I>(&self, index: I) -> (&T::Context, I::Refs<'_, '_>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.index(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index<I>(self, index: I) -> I::Refs<'c, 'a>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.into_index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context<I>(self, index: I) -> (&'c T::Context, I::Refs<'c, 'a>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = T::slices_mut_as_slices(context, slices);
        (context, index.index(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&mut self, index: I) -> I::RefsMut<'_, '_>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.index_mut_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context<I>(&mut self, index: I) -> (&T::Context, I::RefsMut<'_, '_>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        (context, index.index_mut(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut<I>(self, index: I) -> I::RefsMut<'c, 'a>
    where
        I: SoaSlicesIndex<T>,
    {
        let (_, refs) = self.into_index_mut_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut_with_context<I>(self, index: I) -> (&'c T::Context, I::RefsMut<'c, 'a>)
    where
        I: SoaSlicesIndex<T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.index_mut(context, slices))
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, '_, T> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&T::Context, Iter<'_, '_, T>) {
        let (context, iter) = self.raw_iter_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, '_, T> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&mut self) -> (&T::Context, IterMut<'_, '_, T>) {
        let (context, iter) = self.raw_iter_mut_with_context();
        let iter = unsafe { iter.deref_mut() };
        (context, iter)
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c T::Context, IterMut<'c, 'a, T>) {
        let (context, iter) = self.into_raw_iter_mut_with_context();
        let iter = unsafe { iter.deref_mut() };
        (context, iter)
    }

    #[inline]
    pub fn contains<'me, V>(&'me self, value: V) -> bool
    where
        T::Refs<'me, 'me>: PartialEq<V>,
    {
        let mut iter = self.into_iter();
        iter.any(move |item| item.eq(&value))
    }

    #[inline]
    pub fn sort_unstable_with_permutation<P>(&mut self, permutation: P)
    where
        P: AsMut<[usize]>,
        for<'ca, 'any> T::Refs<'ca, 'any>: Ord,
    {
        self.sort_unstable_with_permutation_by(permutation, |a, b| {
            let a = T::upcast_refs(a);
            let b = T::upcast_refs(b);
            Ord::cmp(&a, &b)
        });
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by<P, F>(&mut self, permutation: P, mut compare: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(T::Refs<'_, '_>, T::Refs<'_, '_>) -> cmp::Ordering,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = me.slices().into_parts();
            permutation.sort_unstable_by(|&a, &b| {
                let a = unsafe {
                    let ptrs = context.ptrs_add(ptrs.clone(), a);
                    T::ptrs_to_refs(context, ptrs)
                };
                let b = unsafe {
                    let ptrs = context.ptrs_add(ptrs.clone(), b);
                    T::ptrs_to_refs(context, ptrs)
                };
                compare(a, b)
            });
        });
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by_key<P, K, F>(&mut self, permutation: P, mut f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = me.slices().into_parts();
            permutation.sort_unstable_by_key(|&index| unsafe {
                let ptrs = context.ptrs_add(ptrs.clone(), index);
                let refs = T::ptrs_to_refs(context, ptrs);
                f(refs)
            });
        });
    }
}

impl<T> SoaSlicesMut<'_, '_, T>
where
    T: SoaCloneToUninit + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn clone_from_slices(&mut self, src: &SoaSlices<T>) {
        let len = self.len();
        if len != src.len() {
            len_mismatch_fail(len, src.len());
        }

        for index in 0..len {
            let (context, dst) = unsafe { self.get_unchecked_mut_with_context(index) };
            unsafe { context.ptrs_drop_in_place(dst.clone()) }

            let src = unsafe { src.get_unchecked(index) };
            unsafe { T::clone_to_uninit(context, src, dst) }
        }
    }
}

#[inline(never)]
#[cold]
#[track_caller]
fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
    panic!("source slice length ({src_len}) does not match destination slice length ({dst_len})")
}

impl<'c, T> From<SoaSlicesMut<'c, '_, T>> for SoaSlicePtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlicesMut<'c, '_, T>) -> Self {
        slices.into_slice_ptrs()
    }
}

impl<'c, T> From<SoaSlicesMut<'c, '_, T>> for SoaSliceMutPtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlicesMut<'c, '_, T>) -> Self {
        slices.into_slice_mut_ptrs()
    }
}

impl<'c, 'a, T> From<SoaSlicesMut<'c, 'a, T>> for SoaSlices<'c, 'a, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlicesMut<'c, 'a, T>) -> Self {
        let (context, ptrs, len) = slices.into_parts();
        let ptrs = context.ptrs_cast_const(ptrs);
        unsafe { Self::from_parts(context, ptrs, len) }
    }
}

impl<'c, T> From<&'c T::Context> for SoaSlicesMut<'c, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'c T::Context) -> Self {
        let ptrs = context.ptrs_dangling_mut();
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlicesMut").field(&slices).finish()
    }
}

impl<T> AsRef<Self> for SoaSlicesMut<'_, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Into<&'any [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> AsMut<Self> for SoaSlicesMut<'_, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U> AsMut<[U]> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::SlicesMut<'c, 'any>: Into<&'any mut [U]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices().into()
    }
}

impl<T> Eq for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Eq,
{
}

impl<T> Ord for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let this = self.as_slices();
        let other = other.as_slices();
        Ord::cmp(&this, &other)
    }
}

impl<T> Hash for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let slices = self.as_slices();
        slices.hash(state);
    }
}

impl<T, U, I> Index<I> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    U: ?Sized,
    for<'c, 'any> I: IndexHelper<'c, 'any, T, Output = U>,
{
    type Output = U;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        SoaSlicesMut::index(self, index)
    }
}

impl<T, U, I> IndexMut<I> for SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
    U: ?Sized,
    for<'c, 'any> I: IndexHelperMut<'c, 'any, T, Output = U>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        SoaSlicesMut::index_mut(self, index)
    }
}

impl<'r, T> IntoIterator for &'r SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    type Item = T::Refs<'r, 'r>;
    type IntoIter = Iter<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'r, T> IntoIterator for &'r mut SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    type Item = T::RefsMut<'r, 'r>;
    type IntoIter = IterMut<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'c, 'a, T> IntoIterator for SoaSlicesMut<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    type Item = T::RefsMut<'c, 'a>;
    type IntoIter = IterMut<'c, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
