use core::cmp;
use core_alloc::{borrow::ToOwned, boxed::Box};

use crate::{
    slice::{Iter, IterMut, SoaSlice},
    traits::{AllocSoaTrusted, Refs, RefsMut, Soa, SoaCloneToUninit, SoaRead},
    vec::{IntoIter, SoaVec},
};

impl<T> SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    #[must_use]
    pub fn into_vec(self: Box<Self>) -> SoaVec<T> {
        let len = self.len();
        let capacity = self.capacity();
        let ptr = Box::into_raw(self).cast();
        unsafe { SoaVec::from_raw_parts(ptr, len, capacity) }
    }
}

impl<T> SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    for<'a> T: Soa<'a>,
{
    #[inline]
    pub fn sort_with_permutation<P>(&mut self, permutation: P)
    where
        P: AsMut<[usize]>,
        for<'ctx, 'a> Refs<'ctx, 'a, T>: Ord,
    {
        self.mut_slices().sort_with_permutation(permutation);
    }

    #[inline]
    pub fn sort(&mut self)
    where
        for<'ctx, 'a> Refs<'ctx, 'a, T>: Ord,
    {
        self.mut_slices().sort();
    }

    #[inline]
    pub fn sort_with_permutation_by<P, F>(&mut self, permutation: P, compare: F)
    where
        P: AsMut<[usize]>,
        for<'a> F: FnMut(Refs<'_, 'a, T>, Refs<'_, 'a, T>) -> cmp::Ordering,
    {
        self.mut_slices()
            .sort_with_permutation_by(permutation, compare);
    }

    #[inline]
    pub fn sort_by<F>(&mut self, compare: F)
    where
        for<'a> F: FnMut(Refs<'_, 'a, T>, Refs<'_, 'a, T>) -> cmp::Ordering,
    {
        self.mut_slices().sort_by(compare);
    }

    #[inline]
    pub fn sort_with_permutation_by_key<P, K, F>(&mut self, permutation: P, f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(Refs<'_, '_, T>) -> K,
        K: Ord,
    {
        self.mut_slices()
            .sort_with_permutation_by_key(permutation, f);
    }

    #[inline]
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(Refs<'_, '_, T>) -> K,
        K: Ord,
    {
        self.mut_slices().sort_by_key(f);
    }

    #[inline]
    pub fn sort_with_permutation_by_cached_key<P, K, F>(&mut self, permutation: P, f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(Refs<'_, '_, T>) -> K,
        K: Ord,
    {
        self.mut_slices()
            .sort_with_permutation_by_cached_key(permutation, f);
    }

    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, f: F)
    where
        F: FnMut(Refs<'_, '_, T>) -> K,
        K: Ord,
    {
        self.mut_slices().sort_by_cached_key(f);
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'ctx, 'a> Refs<'ctx, 'a, T>: Ord,
    {
        self.mut_slices().sort_unstable();
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        for<'a> F: FnMut(Refs<'_, 'a, T>, Refs<'_, 'a, T>) -> cmp::Ordering,
    {
        self.mut_slices().sort_unstable_by(compare);
    }

    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(Refs<'_, '_, T>) -> K,
        K: Ord,
    {
        self.mut_slices().sort_unstable_by_key(f);
    }
}

impl<T> SoaSlice<T>
where
    T: AllocSoaTrusted + SoaCloneToUninit + ?Sized,
    T::Context: Clone,
{
    #[inline]
    pub fn to_vec(&self) -> SoaVec<T> {
        self.slices().to_vec()
    }
}

impl<T> ToOwned for SoaSlice<T>
where
    T: AllocSoaTrusted + SoaCloneToUninit + ?Sized,
    T::Context: Clone,
{
    type Owned = SoaVec<T>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.to_vec()
    }

    #[inline]
    fn clone_into(&self, target: &mut Self::Owned) {
        // FIXME: decide if this impl will be better:
        // https://github.com/rust-lang/rust/blob/019fc4de2f3d49a2ef862d180599194d2be05193/library/alloc/src/slice.rs#L860

        target.clear();
        target.extend_from_slice(self);
    }
}

impl<'a, T> IntoIterator for &'a Box<SoaSlice<T>>
where
    T: Soa<'a> + AllocSoaTrusted + ?Sized,
{
    type Item = Refs<'a, 'a, T>;
    type IntoIter = Iter<'a, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Box<SoaSlice<T>>
where
    T: Soa<'a> + AllocSoaTrusted + ?Sized,
{
    type Item = RefsMut<'a, 'a, T>;
    type IntoIter = IterMut<'a, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for Box<SoaSlice<T>>
where
    T: AllocSoaTrusted + SoaRead,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}
