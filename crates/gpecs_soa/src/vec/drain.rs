use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem,
    ops::{Range, RangeBounds},
    ptr::NonNull,
};

use crate::{
    ptr::is_zst,
    slice::{slice_range, Iter},
};

use super::{Soa, SoaVec};

pub struct Drain<'a, T>
where
    T: Soa + 'a,
{
    /// Index of tail to preserve
    tail_start: usize,
    /// Length of tail
    tail_len: usize,
    /// Current remaining range to remove
    iter: Iter<'a, T>,
    vec: NonNull<SoaVec<T>>,
}

impl<'a, T> Drain<'a, T>
where
    T: Soa,
{
    #[inline]
    #[track_caller]
    pub(super) fn new<R>(vec: &'a mut SoaVec<T>, range: R) -> Self
    where
        R: RangeBounds<usize>,
    {
        // Memory safety
        //
        // When the Drain is first created, it shortens the length of
        // the source vector to make sure no uninitialized or moved-from elements
        // are accessible at all if the Drain's destructor never gets to run.
        //
        // Drain will ptr::read out the values to remove.
        // When finished, remaining tail of the vec is copied back to cover
        // the hole, and the vector length is restored to the new length.
        //
        let len = vec.len();
        let range @ Range { start, end } = slice_range(range, ..len);

        unsafe {
            let mut vec = NonNull::from(vec);

            // set self.vec length's to start, to be safe in case Drain is leaked
            vec.as_mut().set_len(start);
            let iter = Iter::from_range(vec.as_ref().as_slice(), range);
            Self {
                tail_start: end,
                tail_len: len - end,
                iter,
                vec,
            }
        }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_> {
        self.iter.as_slices()
    }
}

unsafe impl<T> Send for Drain<'_, T> where T: Soa + Send {}
unsafe impl<T> Sync for Drain<'_, T> where T: Soa + Sync {}

impl<T> Debug for Drain<'_, T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("Drain").field(&slices).finish()
    }
}

impl<T> Iterator for Drain<'_, T>
where
    T: Soa,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|refs| unsafe { T::ptrs_read(T::refs_as_ptrs(refs)) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for Drain<'_, T>
where
    T: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|refs| unsafe { T::ptrs_read(T::refs_as_ptrs(refs)) })
    }
}

impl<T> ExactSizeIterator for Drain<'_, T>
where
    T: Soa,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> FusedIterator for Drain<'_, T> where T: Soa {}

impl<T> Drop for Drain<'_, T>
where
    T: Soa,
{
    fn drop(&mut self) {
        /// Moves back the un-`Drain`ed elements to restore the original `Vec`.
        struct DropGuard<'r, 'a, T>(&'r mut Drain<'a, T>)
        where
            T: Soa;

        impl<T> Drop for DropGuard<'_, '_, T>
        where
            T: Soa,
        {
            fn drop(&mut self) {
                if self.0.tail_len == 0 {
                    return;
                }
                unsafe {
                    let source_vec = self.0.vec.as_mut();
                    // memory-move back untouched tail, update to new length
                    let start = source_vec.len();
                    let tail = self.0.tail_start;
                    if tail != start {
                        let src = T::ptrs_add(source_vec.as_ptrs(), tail);
                        let dst = T::ptrs_add_mut(source_vec.as_mut_ptrs(), start);
                        T::ptrs_copy(src, dst, self.0.tail_len);
                    }
                    source_vec.set_len(start + self.0.tail_len);
                }
            }
        }

        let iter = mem::take(&mut self.iter);
        let drop_len = iter.len();

        let mut vec = self.vec;

        if is_zst::<T>() {
            // ZSTs have no identity, so we don't need to move them around, we only need to drop the correct amount.
            // this can be achieved by manipulating the Vec length instead of moving values out from `iter`.
            unsafe {
                let vec = vec.as_mut();
                let old_len = vec.len();
                vec.set_len(old_len + drop_len + self.tail_len);
                vec.truncate(old_len + self.tail_len);
            }

            return;
        }

        // ensure elements are moved back into their appropriate places, even when drop_in_place panics
        let _guard = DropGuard(self);

        if drop_len == 0 {
            return;
        }

        // as_ptrs() must only be called when iter.len() is > 0 because
        // it also gets touched by vec::Splice which may turn it into a dangling pointer
        // which would make it and the vec pointer point to different allocations which would
        // lead to invalid pointer arithmetic below.
        let drop_ptrs = T::slice_refs_as_ptrs(iter.as_slices());

        unsafe {
            // drop_ptrs comes from an Iter which only gives us slices but for drop_in_place
            // a pointer with mutable provenance is necessary. Therefore we must reconstruct
            // it from the original vec but also avoid creating a &mut to the front since that could
            // invalidate raw pointers to it which some unsafe code might rely on.
            let vec_ptrs = vec.as_mut().as_mut_ptrs();
            let drop_offset = T::ptrs_offset_from(drop_ptrs, T::ptrs_cast_const(vec_ptrs.clone()))
                .try_into()
                .unwrap_unchecked();
            let to_drop =
                T::slices_from_raw_parts_mut(T::ptrs_add_mut(vec_ptrs, drop_offset), drop_len);
            T::slices_drop_in_place(to_drop);
        }
    }
}
