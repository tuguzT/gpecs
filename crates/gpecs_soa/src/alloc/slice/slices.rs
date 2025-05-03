use core::cmp;
use core_alloc::boxed::Box;

use crate::{
    alloc::set_len_on_drop::SetLenOnDrop,
    slice::{SoaSlices, SoaSlicesMut},
    traits::{Soa, SoaToOwned},
    vec::SoaVec,
};

impl<'c, 'a, T> SoaSlices<'c, 'a, T>
where
    T: Soa,
{
    #[inline]
    pub fn to_vec(&self) -> SoaVec<T>
    where
        T::Refs<'c, 'a>: SoaToOwned<'c, 'a, Owned = T>,
        T::Context: Clone,
    {
        let len = self.len();
        let context = self.context().clone();
        let mut vec = SoaVec::with_context_and_capacity(context, len);

        {
            let mut set_len_on_drop = SetLenOnDrop {
                vec: &mut vec,
                local_len: 0,
            };
            let ptrs: T::MutPtrs<'_> = set_len_on_drop.vec.buffer.ptrs();
            let context = set_len_on_drop.vec.context();
            for (index, refs) in self.clone().into_iter().enumerate() {
                set_len_on_drop.local_len = index;
                unsafe {
                    let dst = T::ptrs_add_mut(context, ptrs.clone(), index);
                    refs.clone_into_ptrs(context, dst);
                }
            }
        }

        // SAFETY:
        // the vec was allocated and initialized above to at least this length.
        unsafe {
            vec.set_len(len);
        }
        vec
    }
}

impl<'c, 'a, T> SoaSlicesMut<'c, 'a, T>
where
    T: Soa,
{
    #[inline]
    pub fn to_vec(&self) -> SoaVec<T>
    where
        T::Refs<'c, 'a>: SoaToOwned<'c, 'a, Owned = T>,
        T::Context: Clone,
    {
        let len = self.len();
        let context = self.context().clone();
        let mut vec = SoaVec::with_context_and_capacity(context, len);

        {
            let mut set_len_on_drop = SetLenOnDrop {
                vec: &mut vec,
                local_len: 0,
            };
            let ptrs: T::MutPtrs<'_> = set_len_on_drop.vec.buffer.ptrs();
            let context = set_len_on_drop.vec.context();

            let slices = {
                let (context, ptrs, len) = unsafe { self.as_parts() };
                let ptrs = T::ptrs_cast_const(context, ptrs.clone());
                unsafe { SoaSlices::<T>::from_parts(context, ptrs, len) }
            };
            for (index, refs) in slices.into_iter().enumerate() {
                set_len_on_drop.local_len = index;
                unsafe {
                    let dst = T::ptrs_add_mut(context, ptrs.clone(), index);
                    refs.clone_into_ptrs(context, dst);
                }
            }
        }

        // SAFETY:
        // the vec was allocated and initialized above to at least this length.
        unsafe {
            vec.set_len(len);
        }
        vec
    }

    #[inline]
    pub fn sort(&mut self)
    where
        for<'ca, 'any> T::Refs<'ca, 'any>: Ord,
    {
        let mut permutation = (0..self.len()).collect::<Box<_>>();
        self.sort_with_permutation(&mut permutation)
    }

    #[inline]
    pub fn sort_with_permutation(&mut self, permutation: &mut [usize])
    where
        for<'ca, 'any> T::Refs<'ca, 'any>: Ord,
    {
        self.sort_with_permutation_by(permutation, |a, b| Ord::cmp(&a, &b))
    }

    #[inline]
    pub fn sort_by<F>(&mut self, compare: F)
    where
        for<'ca, 'any> F: FnMut(T::Refs<'ca, 'any>, T::Refs<'ca, 'any>) -> cmp::Ordering,
    {
        let mut permutation = (0..self.len()).collect::<Box<_>>();
        self.sort_with_permutation_by(&mut permutation, compare)
    }

    #[inline]
    pub fn sort_with_permutation_by<F>(&mut self, permutation: &mut [usize], mut compare: F)
    where
        for<'ca, 'any> F: FnMut(T::Refs<'ca, 'any>, T::Refs<'ca, 'any>) -> cmp::Ordering,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = unsafe { me.as_parts() };
            permutation.sort_by(|&a, &b| {
                let a = unsafe {
                    let ptrs = T::ptrs_add_mut(context, ptrs.clone(), a);
                    let ptrs = T::ptrs_cast_const(context, ptrs);
                    T::ptrs_to_refs(context, ptrs)
                };
                let b = unsafe {
                    let ptrs = T::ptrs_add_mut(context, ptrs.clone(), b);
                    let ptrs = T::ptrs_cast_const(context, ptrs);
                    T::ptrs_to_refs(context, ptrs)
                };
                compare(a, b)
            })
        })
    }

    #[inline]
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        let mut permutation = (0..self.len()).collect::<Box<_>>();
        self.sort_with_permutation_by_key(&mut permutation, f)
    }

    #[inline]
    pub fn sort_with_permutation_by_key<K, F>(&mut self, permutation: &mut [usize], mut f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = unsafe { me.as_parts() };
            permutation.sort_by_key(|&index| unsafe {
                let ptrs = T::ptrs_add_mut(context, ptrs.clone(), index);
                let ptrs = T::ptrs_cast_const(context, ptrs);
                let refs = T::ptrs_to_refs(context, ptrs);
                f(refs)
            })
        })
    }

    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        let mut permutation = (0..self.len()).collect::<Box<_>>();
        self.sort_with_permutation_by_cached_key(&mut permutation, f)
    }

    #[inline]
    pub fn sort_with_permutation_by_cached_key<K, F>(&mut self, permutation: &mut [usize], mut f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = unsafe { me.as_parts() };
            permutation.sort_by_cached_key(|&index| unsafe {
                let ptrs = T::ptrs_add_mut(context, ptrs.clone(), index);
                let ptrs = T::ptrs_cast_const(context, ptrs);
                let refs = T::ptrs_to_refs(context, ptrs);
                f(refs)
            })
        })
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'ca, 'any> T::Refs<'ca, 'any>: Ord,
    {
        let mut permutation = (0..self.len()).collect::<Box<_>>();
        self.sort_unstable_with_permutation(&mut permutation)
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        for<'ca, 'any> F: FnMut(T::Refs<'ca, 'any>, T::Refs<'ca, 'any>) -> cmp::Ordering,
    {
        let mut permutation = (0..self.len()).collect::<Box<_>>();
        self.sort_unstable_with_permutation_by(&mut permutation, compare)
    }

    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        let mut permutation = (0..self.len()).collect::<Box<_>>();
        self.sort_unstable_with_permutation_by_key(&mut permutation, f)
    }
}
