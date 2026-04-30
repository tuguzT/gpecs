use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    slice::{
        IndexHelper, IndexHelperMut, Iter, IterMut, RawIter, RawIterMut, SoaSliceMutPtrs,
        SoaSlicePtrs, SoaSlicePtrsIndex, SoaSlices, SoaSlicesIndex, assert::slice_index_usize_fail,
    },
    traits::{
        CloneToUninitSoaContext, MutPtrs, Ptrs, RawSoa, RawSoaContext, Refs, RefsMut, SliceMutPtrs,
        SlicePtrs, Slices, SlicesMut, Soa, SoaCloneToUninit, SoaContext, SoaOwned,
    },
};

pub struct SoaSlicesMut<'ctx, 'a, T>
where
    T: RawSoa + ?Sized,
{
    ptrs: SoaSliceMutPtrs<'ctx, T>,
    phantom: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'ctx, 'a, T> SoaSlicesMut<'ctx, 'a, T>
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
    pub fn as_mut_slice_ptrs(&mut self) -> SliceMutPtrs<'_, T> {
        let (_, ptrs) = self.as_mut_slice_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(&mut self) -> (&T::Context, SliceMutPtrs<'_, T>) {
        let Self { ptrs, .. } = self;
        ptrs.as_mut_slice_ptrs_with_context()
    }

    #[inline]
    pub fn slice_ptrs(&self) -> SoaSlicePtrs<'_, T> {
        let Self { ptrs, .. } = self;
        ptrs.clone().cast_const()
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SoaSlicePtrs<'ctx, T> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn mut_slice_ptrs(&mut self) -> SoaSliceMutPtrs<'_, T> {
        let Self { ptrs, .. } = self;
        ptrs.clone()
    }

    #[inline]
    pub fn into_mut_slice_ptrs(self) -> SoaSliceMutPtrs<'ctx, T> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, '_, T> {
        let (_, slices) = self.slices_with_context();
        slices
    }

    #[inline]
    pub fn slices_with_context(&self) -> (&T::Context, SoaSlices<'_, '_, T>) {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        let (context, ptrs) = ptrs.as_ptrs_with_context();
        let slices = unsafe { SoaSlices::from_parts(context, ptrs, len) };
        (context, slices)
    }

    #[inline]
    pub fn mut_slices(&mut self) -> SoaSlicesMut<'_, '_, T> {
        let (_, slices) = self.mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn mut_slices_with_context(&mut self) -> (&T::Context, SoaSlicesMut<'_, '_, T>) {
        let Self { ptrs, .. } = self;

        let len = ptrs.len();
        let (context, ptrs) = ptrs.as_mut_ptrs_with_context();
        let slices = unsafe { SoaSlicesMut::from_parts(context, ptrs, len) };
        (context, slices)
    }

    #[inline]
    pub fn into_parts(self) -> (&'ctx T::Context, MutPtrs<'ctx, T>, usize) {
        let Self { ptrs, .. } = self;
        ptrs.into_parts()
    }

    #[inline]
    pub unsafe fn from_parts(
        context: &'ctx T::Context,
        ptrs: MutPtrs<'ctx, T>,
        len: usize,
    ) -> Self {
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
    pub unsafe fn into_get_unchecked<I>(self, index: I) -> I::Ptrs<'ctx>
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
    ) -> (&'ctx T::Context, I::Ptrs<'ctx>)
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
    pub unsafe fn into_get_unchecked_mut<I>(self, index: I) -> I::MutPtrs<'ctx>
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
    ) -> (&'ctx T::Context, I::MutPtrs<'ctx>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.into_get_unchecked_mut_with_context(index) }
    }

    #[inline]
    pub unsafe fn split_at_unchecked(
        self,
        mid: usize,
    ) -> (SoaSlices<'ctx, 'a, T>, SoaSlices<'ctx, 'a, T>) {
        let Self { ptrs, .. } = self;

        let (first, second) = unsafe { ptrs.clone().split_at_unchecked(mid) };
        unsafe { (first.as_ref_unchecked(), second.as_ref_unchecked()) }
    }

    #[inline]
    pub fn split_at_checked(
        self,
        mid: usize,
    ) -> Option<(SoaSlices<'ctx, 'a, T>, SoaSlices<'ctx, 'a, T>)> {
        if mid <= self.len() {
            // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
            // fulfills the requirements of `split_at_unchecked`.
            Some(unsafe { self.split_at_unchecked(mid) })
        } else {
            None
        }
    }

    #[inline]
    #[track_caller]
    pub fn split_at(self, mid: usize) -> (SoaSlices<'ctx, 'a, T>, SoaSlices<'ctx, 'a, T>) {
        match self.split_at_checked(mid) {
            Some(pair) => pair,
            None => panic!("mid > len"),
        }
    }

    #[inline]
    pub unsafe fn split_at_mut_unchecked(
        self,
        mid: usize,
    ) -> (SoaSlicesMut<'ctx, 'a, T>, SoaSlicesMut<'ctx, 'a, T>) {
        let Self { ptrs, .. } = self;

        let (first, second) = unsafe { ptrs.clone().split_at_mut_unchecked(mid) };
        unsafe { (first.as_mut_unchecked(), second.as_mut_unchecked()) }
    }

    #[inline]
    pub fn split_at_mut_checked(
        self,
        mid: usize,
    ) -> Option<(SoaSlicesMut<'ctx, 'a, T>, SoaSlicesMut<'ctx, 'a, T>)> {
        if mid <= self.len() {
            // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
            // fulfills the requirements of `split_at_unchecked`.
            Some(unsafe { self.split_at_mut_unchecked(mid) })
        } else {
            None
        }
    }

    #[inline]
    #[track_caller]
    pub fn split_at_mut(
        self,
        mid: usize,
    ) -> (SoaSlicesMut<'ctx, 'a, T>, SoaSlicesMut<'ctx, 'a, T>) {
        match self.split_at_mut_checked(mid) {
            Some(pair) => pair,
            None => panic!("mid > len"),
        }
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
    pub fn into_raw_iter(self) -> RawIter<'ctx, T> {
        let (_, iter) = self.into_raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_with_context(self) -> (&'ctx T::Context, RawIter<'ctx, T>) {
        let (context, iter) = self.into_raw_iter_mut_with_context();
        (context, iter.cast_const())
    }

    #[inline]
    pub fn into_raw_iter_mut(self) -> RawIterMut<'ctx, T> {
        let (_, iter) = self.into_raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_mut_with_context(self) -> (&'ctx T::Context, RawIterMut<'ctx, T>) {
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

        unsafe { ptrs.swap_unchecked(a, b) }
    }

    #[inline]
    pub unsafe fn swap_unchecked(&mut self, a: usize, b: usize) {
        let Self { ptrs, .. } = self;
        unsafe { ptrs.swap_unchecked(a, b) }
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

        if len < 2 {
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

impl<'ctx, 'a, T> SoaSlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn new(context: &'ctx T::Context, slices: SlicesMut<'ctx, 'a, T>) -> Self {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        Self {
            ptrs: SoaSliceMutPtrs::new(context, slices),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_slices(self) -> SlicesMut<'ctx, 'a, T> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'ctx T::Context, SlicesMut<'ctx, 'a, T>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.into_mut_slice_ptrs_with_context();
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        (context, slices)
    }

    #[inline]
    pub fn into_get<I>(self, index: I) -> Option<I::Refs<'ctx>>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.into_get_with_context(index);
        refs
    }

    #[inline]
    pub fn into_get_with_context<I>(self, index: I) -> (&'ctx T::Context, Option<I::Refs<'ctx>>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = context.mut_slices_as_slices(slices);
        (context, index.get(context, slices))
    }

    #[inline]
    pub fn into_get_mut<I>(self, index: I) -> Option<I::RefsMut<'ctx>>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.into_get_mut_with_context(index);
        refs
    }

    #[inline]
    pub fn into_get_mut_with_context<I>(
        self,
        index: I,
    ) -> (&'ctx T::Context, Option<I::RefsMut<'ctx>>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.get_mut(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index<I>(self, index: I) -> I::Refs<'ctx>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.into_index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context<I>(self, index: I) -> (&'ctx T::Context, I::Refs<'ctx>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.into_slices_with_context();
        let slices = context.mut_slices_as_slices(slices);
        (context, index.index(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut<I>(self, index: I) -> I::RefsMut<'ctx>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.into_index_mut_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut_with_context<I>(self, index: I) -> (&'ctx T::Context, I::RefsMut<'ctx>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.into_slices_with_context();
        (context, index.index_mut(context, slices))
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'ctx T::Context, IterMut<'ctx, 'a, T>) {
        let (context, iter) = self.into_raw_iter_mut_with_context();
        let iter = unsafe { iter.as_mut_unchecked() };
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_iter_with_context(
        self,
    ) -> (&'ctx T::Context, crate::slice::ParIterMut<'ctx, 'a, T>) {
        let (context, _, _) = self.ptrs.clone().into_parts();
        let iter = crate::slice::ParIterMut::new(self);
        (context, iter)
    }
}

impl<'a, T> SoaSlicesMut<'_, '_, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> Slices<'a, 'a, T> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a T::Context, Slices<'a, 'a, T>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.as_slice_ptrs_with_context();
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_mut_slices(&'a mut self) -> SlicesMut<'a, 'a, T> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&'a mut self) -> (&'a T::Context, SlicesMut<'a, 'a, T>) {
        let Self { ptrs, .. } = self;

        let (context, slices) = ptrs.as_mut_slice_ptrs_with_context();
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        (context, slices)
    }

    #[inline]
    pub fn get<I>(&'a self, index: I) -> Option<I::Refs<'a>>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.get_with_context(index);
        refs
    }

    #[inline]
    pub fn get_with_context<I>(&'a self, index: I) -> (&'a T::Context, Option<I::Refs<'a>>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.get(context, slices))
    }

    #[inline]
    pub fn get_mut<I>(&'a mut self, index: I) -> Option<I::RefsMut<'a>>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.get_mut_with_context(index);
        refs
    }

    #[inline]
    pub fn get_mut_with_context<I>(
        &'a mut self,
        index: I,
    ) -> (&'a T::Context, Option<I::RefsMut<'a>>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        (context, index.get_mut(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&'a self, index: I) -> I::Refs<'a>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context<I>(&'a self, index: I) -> (&'a T::Context, I::Refs<'a>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.as_slices_with_context();
        (context, index.index(context, slices))
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&'a mut self, index: I) -> I::RefsMut<'a>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.index_mut_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context<I>(&'a mut self, index: I) -> (&'a T::Context, I::RefsMut<'a>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (context, slices) = self.as_mut_slices_with_context();
        (context, index.index_mut(context, slices))
    }

    #[inline]
    pub fn iter(&'a self) -> Iter<'a, 'a, T> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&'a self) -> (&'a T::Context, Iter<'a, 'a, T>) {
        let (context, iter) = self.raw_iter_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
        (context, iter)
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> IterMut<'a, 'a, T> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&'a mut self) -> (&'a T::Context, IterMut<'a, 'a, T>) {
        let (context, iter) = self.raw_iter_mut_with_context();
        let iter = unsafe { iter.as_mut_unchecked() };
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter(&'a self) -> crate::slice::ParIter<'a, 'a, T> {
        let (_, iter) = self.par_iter_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_with_context(&'a self) -> (&'a T::Context, crate::slice::ParIter<'a, 'a, T>) {
        let (context, slices) = self.slices_with_context();
        let iter = crate::slice::ParIter::new(slices);
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_mut(&'a mut self) -> crate::slice::ParIterMut<'a, 'a, T> {
        let (_, iter) = self.par_iter_mut_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_mut_with_context(
        &'a mut self,
    ) -> (&'a T::Context, crate::slice::ParIterMut<'a, 'a, T>) {
        let (context, slices) = self.mut_slices_with_context();
        let iter = crate::slice::ParIterMut::new(slices);
        (context, iter)
    }

    #[inline]
    pub fn contains<V>(&'a self, value: V) -> bool
    where
        Refs<'a, 'a, T>: PartialEq<V>,
    {
        let mut iter = self.into_iter();
        iter.any(move |item| item.eq(&value))
    }
}

impl<T> SoaSlicesMut<'_, '_, T>
where
    T: SoaOwned + ?Sized,
{
    #[inline]
    pub fn sort_unstable_with_permutation<P>(&mut self, permutation: P)
    where
        P: AsMut<[usize]>,
        for<'ctx, 'a> Refs<'ctx, 'a, T>: Ord,
    {
        self.sort_unstable_with_permutation_by(permutation, |a, b| {
            let a = T::Context::upcast_refs(a);
            let b = T::Context::upcast_refs(b);
            Ord::cmp(&a, &b)
        });
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by<P, F>(&mut self, permutation: P, mut compare: F)
    where
        P: AsMut<[usize]>,
        for<'a> F: FnMut(Refs<'_, 'a, T>, Refs<'_, 'a, T>) -> cmp::Ordering,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = me.slices().into_parts();
            permutation.sort_unstable_by(|&a, &b| {
                let a = unsafe {
                    let ptrs = context.ptrs_add(ptrs.clone(), a);
                    context.ptrs_to_refs(ptrs)
                };
                let b = unsafe {
                    let ptrs = context.ptrs_add(ptrs.clone(), b);
                    context.ptrs_to_refs(ptrs)
                };
                compare(a, b)
            });
        });
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by_key<P, K, F>(&mut self, permutation: P, mut f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(Refs<'_, '_, T>) -> K,
        K: Ord,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = me.slices().into_parts();
            permutation.sort_unstable_by_key(|&index| unsafe {
                let ptrs = context.ptrs_add(ptrs.clone(), index);
                let refs = context.ptrs_to_refs(ptrs);
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
            unsafe { context.clone_to_uninit(src, dst) }
        }
    }
}

#[inline(never)]
#[cold]
#[track_caller]
fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
    panic!("source slice length ({src_len}) does not match destination slice length ({dst_len})")
}

impl<'ctx, T> From<SoaSlicesMut<'ctx, '_, T>> for SoaSlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlicesMut<'ctx, '_, T>) -> Self {
        slices.into_slice_ptrs()
    }
}

impl<'ctx, T> From<SoaSlicesMut<'ctx, '_, T>> for SoaSliceMutPtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlicesMut<'ctx, '_, T>) -> Self {
        slices.into_mut_slice_ptrs()
    }
}

impl<'ctx, 'a, T> From<SoaSlicesMut<'ctx, 'a, T>> for SoaSlices<'ctx, 'a, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(slices: SoaSlicesMut<'ctx, 'a, T>) -> Self {
        let (context, ptrs, len) = slices.into_parts();
        let ptrs = context.ptrs_cast_const(ptrs);
        unsafe { Self::from_parts(context, ptrs, len) }
    }
}

impl<'ctx, T> From<&'ctx T::Context> for SoaSlicesMut<'ctx, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'ctx T::Context) -> Self {
        let ptrs = context.ptrs_dangling_mut();
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSlicesMut<'_, '_, T>
where
    T: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Debug,
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
    T: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Into<&'a [U]>,
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
    T: SoaOwned + ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, T>: Into<&'a mut [U]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices().into()
    }
}

impl<T> Eq for SoaSlicesMut<'_, '_, T>
where
    T: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Eq,
{
}

impl<T> Ord for SoaSlicesMut<'_, '_, T>
where
    T: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Ord,
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
    T: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let slices = self.as_slices();
        slices.hash(state);
    }
}

impl<T, U, I> Index<I> for SoaSlicesMut<'_, '_, T>
where
    T: SoaOwned + ?Sized,
    U: ?Sized,
    for<'ctx, 'a> I: IndexHelper<'ctx, 'a, T, Output = U>,
{
    type Output = U;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        SoaSlicesMut::index(self, index)
    }
}

impl<T, U, I> IndexMut<I> for SoaSlicesMut<'_, '_, T>
where
    T: SoaOwned + ?Sized,
    U: ?Sized,
    for<'ctx, 'a> I: IndexHelperMut<'ctx, 'a, T, Output = U>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        SoaSlicesMut::index_mut(self, index)
    }
}

impl<'a, T> IntoIterator for &'a SoaSlicesMut<'_, '_, T>
where
    T: Soa<'a> + ?Sized,
{
    type Item = Refs<'a, 'a, T>;
    type IntoIter = Iter<'a, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SoaSlicesMut<'_, '_, T>
where
    T: Soa<'a> + ?Sized,
{
    type Item = RefsMut<'a, 'a, T>;
    type IntoIter = IterMut<'a, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'ctx, 'a, T> IntoIterator for SoaSlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    type Item = RefsMut<'ctx, 'a, T>;
    type IntoIter = IterMut<'ctx, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}

#[cfg(feature = "rayon")]
impl<'a, T> rayon::iter::IntoParallelIterator for &'a SoaSlicesMut<'_, '_, T>
where
    T: Soa<'a> + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
    Refs<'a, 'a, T>: Send,
{
    type Item = Refs<'a, 'a, T>;
    type Iter = crate::slice::ParIter<'a, 'a, T>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, T> rayon::iter::IntoParallelIterator for &'a mut SoaSlicesMut<'_, '_, T>
where
    T: Soa<'a> + ?Sized,
    T::Context: Sync,
    T::Fields: Send,
    RefsMut<'a, 'a, T>: Send,
{
    type Item = RefsMut<'a, 'a, T>;
    type Iter = crate::slice::ParIterMut<'a, 'a, T>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter_mut()
    }
}

#[cfg(feature = "rayon")]
impl<'ctx, 'a, T> rayon::iter::IntoParallelIterator for SoaSlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    T::Context: Sync,
    T::Fields: Send,
    RefsMut<'ctx, 'a, T>: Send,
{
    type Item = RefsMut<'ctx, 'a, T>;
    type Iter = crate::slice::ParIterMut<'ctx, 'a, T>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        let (_, iter) = self.into_par_iter_with_context();
        iter
    }
}

unsafe impl<T> Send for SoaSlicesMut<'_, '_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Sync,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for SoaSlicesMut<'_, '_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}
