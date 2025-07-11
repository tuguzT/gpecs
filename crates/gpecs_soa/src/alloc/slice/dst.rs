use core::{cmp, ptr};
use core_alloc::{borrow::ToOwned, boxed::Box};

use crate::{
    ptr::{BufferData, slice_from_raw_parts_mut},
    slice::{Iter, IterMut, SoaSlice},
    traits::{SoaToOwned, SoaTrustedFields},
    vec::{IntoIter, SoaVec},
};

impl<T> SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    pub fn into_vec(self: Box<Self>) -> SoaVec<T> {
        let len = self.len();
        let capacity = self.capacity();
        let ptr = Box::into_raw(self).cast();
        unsafe { SoaVec::from_raw_parts(ptr, len, capacity) }
    }

    #[inline]
    pub fn to_vec<'me>(&'me self) -> SoaVec<T>
    where
        for<'c, 'any> T::Refs<'c, 'any>: SoaToOwned<'c, 'any, Owned = T>,
        T::Context: Clone,
    {
        self.slices().to_vec()
    }

    #[inline]
    pub fn sort_with_permutation(&mut self, permutation: &mut [usize])
    where
        for<'c, 'any> T::Refs<'c, 'any>: Ord,
    {
        self.slices_mut().sort_with_permutation(permutation);
    }

    #[inline]
    pub fn sort(&mut self)
    where
        for<'c, 'any> T::Refs<'c, 'any>: Ord,
    {
        self.slices_mut().sort();
    }

    #[inline]
    pub fn sort_with_permutation_by<F>(&mut self, permutation: &mut [usize], compare: F)
    where
        for<'c, 'any> F: FnMut(T::Refs<'c, 'any>, T::Refs<'c, 'any>) -> cmp::Ordering,
    {
        self.slices_mut()
            .sort_with_permutation_by(permutation, compare);
    }

    #[inline]
    pub fn sort_by<F>(&mut self, compare: F)
    where
        for<'c, 'any> F: FnMut(T::Refs<'c, 'any>, T::Refs<'c, 'any>) -> cmp::Ordering,
    {
        self.slices_mut().sort_by(compare);
    }

    #[inline]
    pub fn sort_with_permutation_by_key<K, F>(&mut self, permutation: &mut [usize], f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.slices_mut()
            .sort_with_permutation_by_key(permutation, f);
    }

    #[inline]
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.slices_mut().sort_by_key(f);
    }

    #[inline]
    pub fn sort_with_permutation_by_cached_key<K, F>(&mut self, permutation: &mut [usize], f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.slices_mut()
            .sort_with_permutation_by_cached_key(permutation, f);
    }

    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.slices_mut().sort_by_cached_key(f);
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'c, 'any> T::Refs<'c, 'any>: Ord,
    {
        self.slices_mut().sort_unstable();
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        for<'c, 'any> F: FnMut(T::Refs<'c, 'any>, T::Refs<'c, 'any>) -> cmp::Ordering,
    {
        self.slices_mut().sort_unstable_by(compare);
    }

    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.slices_mut().sort_unstable_by_key(f);
    }
}

impl<T> Default for Box<SoaSlice<T>>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn default() -> Self {
        let data = ptr::dangling_mut::<BufferData<T>>();
        unsafe { Box::from_raw(slice_from_raw_parts_mut(data, 0, 0)) }
    }
}

impl<T> ToOwned for SoaSlice<T>
where
    T: SoaTrustedFields,
    T::Context: Clone,
    for<'c, 'any> T::Refs<'c, 'any>: SoaToOwned<'c, 'any, Owned = T> + 'any,
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

impl<'r, T> IntoIterator for &'r Box<SoaSlice<T>>
where
    T: SoaTrustedFields,
{
    type Item = T::Refs<'r, 'r>;
    type IntoIter = Iter<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'r, T> IntoIterator for &'r mut Box<SoaSlice<T>>
where
    T: SoaTrustedFields,
{
    type Item = T::RefsMut<'r, 'r>;
    type IntoIter = IterMut<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for Box<SoaSlice<T>>
where
    T: SoaTrustedFields,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}
