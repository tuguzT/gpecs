use core::cmp;
use core_alloc::boxed::Box;

use crate::{
    alloc::set_len_on_drop::SetLenOnDrop,
    slice::{SoaSlices, SoaSlicesMut},
    traits::{RawSoaContext, Soa, SoaCloneToUninit},
    vec::SoaVec,
};

impl<T> SoaSlices<'_, '_, T>
where
    T: SoaCloneToUninit + ?Sized,
    T::Context: Clone,
{
    #[inline]
    pub fn to_vec(&self) -> SoaVec<T> {
        let len = self.len();
        let context = self.context().clone();
        let mut vec = SoaVec::<T>::with_context_and_capacity(context, len);

        {
            let mut set_len_on_drop = SetLenOnDrop {
                vec: &mut vec,
                local_len: 0,
            };

            let (context, dst, _) = set_len_on_drop.vec.slices_mut().into_parts();
            for (index, src) in self.raw_iter().enumerate() {
                set_len_on_drop.local_len = index;

                let dst = unsafe { context.ptrs_add_mut(dst.clone(), index) };
                unsafe { T::clone_to_uninit(context, src, dst) }
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

impl<T> SoaSlicesMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn sort(&mut self)
    where
        for<'c, 'any> T::Refs<'c, 'any>: Ord,
    {
        let permutation = alloc_permutation(self.len());
        self.sort_with_permutation(permutation);
    }

    #[inline]
    pub fn sort_with_permutation<P>(&mut self, permutation: P)
    where
        P: AsMut<[usize]>,
        for<'c, 'any> T::Refs<'c, 'any>: Ord,
    {
        self.sort_with_permutation_by(permutation, |a, b| Ord::cmp(&a, &b));
    }

    #[inline]
    pub fn sort_by<F>(&mut self, compare: F)
    where
        for<'c, 'any> F: FnMut(T::Refs<'c, 'any>, T::Refs<'c, 'any>) -> cmp::Ordering,
    {
        let permutation = alloc_permutation(self.len());
        self.sort_with_permutation_by(permutation, compare);
    }

    #[inline]
    pub fn sort_with_permutation_by<P, F>(&mut self, permutation: P, mut compare: F)
    where
        P: AsMut<[usize]>,
        for<'c, 'any> F: FnMut(T::Refs<'c, 'any>, T::Refs<'c, 'any>) -> cmp::Ordering,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = me.slices().into_parts();
            permutation.sort_by(|&a, &b| {
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
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        let permutation = alloc_permutation(self.len());
        self.sort_with_permutation_by_key(permutation, f);
    }

    #[inline]
    pub fn sort_with_permutation_by_key<P, K, F>(&mut self, permutation: P, mut f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = me.slices().into_parts();
            permutation.sort_by_key(|&index| unsafe {
                let ptrs = context.ptrs_add(ptrs.clone(), index);
                let refs = T::ptrs_to_refs(context, ptrs);
                f(refs)
            });
        });
    }

    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        let permutation = alloc_permutation(self.len());
        self.sort_with_permutation_by_cached_key(permutation, f);
    }

    #[inline]
    pub fn sort_with_permutation_by_cached_key<P, K, F>(&mut self, permutation: P, mut f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.sort_impl(permutation, |me, permutation| {
            let (context, ptrs, _) = me.slices().into_parts();
            permutation.sort_by_cached_key(|&index| unsafe {
                let ptrs = context.ptrs_add(ptrs.clone(), index);
                let refs = T::ptrs_to_refs(context, ptrs);
                f(refs)
            });
        });
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'c, 'any> T::Refs<'c, 'any>: Ord,
    {
        let permutation = alloc_permutation(self.len());
        self.sort_unstable_with_permutation(permutation);
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        for<'c, 'any> F: FnMut(T::Refs<'c, 'any>, T::Refs<'c, 'any>) -> cmp::Ordering,
    {
        let permutation = alloc_permutation(self.len());
        self.sort_unstable_with_permutation_by(permutation, compare);
    }

    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        let permutation = alloc_permutation(self.len());
        self.sort_unstable_with_permutation_by_key(permutation, f);
    }
}

impl<T> SoaSlicesMut<'_, '_, T>
where
    T: SoaCloneToUninit + ?Sized,
    T::Context: Clone,
{
    #[inline]
    pub fn to_vec(&self) -> SoaVec<T> {
        self.slices().to_vec()
    }
}

#[inline]
fn alloc_permutation(len: usize) -> Box<[usize]> {
    (0..len).collect()
}
