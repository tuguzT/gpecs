use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::{Range, RangeBounds},
    ptr::NonNull,
};

use crate::{
    layout::is_zst,
    slice::{Iter, range},
    traits::{AllocSoa, Ptrs, RawSoaContext, SlicePtrs, Slices, Soa, SoaOwned, SoaRead},
};

use super::SoaVec;

pub struct Drain<'a, T>
where
    T: AllocSoa + ?Sized + 'a,
{
    /// Index of tail to preserve
    tail_start: usize,
    /// Length of tail
    tail_len: usize,
    /// Current remaining range to remove
    iter: Iter<'a, 'a, T>,
    vec: NonNull<SoaVec<T>>,
}

impl<'a, T> Drain<'a, T>
where
    T: AllocSoa + ?Sized,
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
        let range @ Range { start, end } = self::range(range, ..len);

        let mut vec = NonNull::from_mut(vec);
        // index before setting length, otherwise range is invalid
        let slices = unsafe { vec.as_ref() }.slice_ptrs();
        let (context, slices) = unsafe { slices.into_get_unchecked_with_context(range) };
        unsafe {
            // set self.vec length's to start, to be safe in case Drain is leaked
            vec.as_mut().set_len(start);
        }

        Self {
            tail_start: end,
            tail_len: len - end,
            iter: unsafe { Iter::from_parts(context, slices) },
            vec,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { iter, .. } = self;
        iter.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { iter, .. } = self;
        iter.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let Self { iter, .. } = self;
        iter.as_ptrs_with_context()
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, T> {
        let (_, ptrs) = self.as_slice_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&T::Context, SlicePtrs<'_, T>) {
        let Self { iter, .. } = self;
        iter.as_slice_ptrs_with_context()
    }
}

impl<'a, T> Drain<'_, T>
where
    T: AllocSoa + Soa<'a> + ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> Slices<'a, 'a, T> {
        let (_, iter) = self.as_slices_with_context();
        iter
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a T::Context, Slices<'a, 'a, T>) {
        let Self { iter, .. } = self;
        iter.as_slices_with_context()
    }
}

unsafe impl<T> Send for Drain<'_, T>
where
    T: AllocSoa + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for Drain<'_, T>
where
    T: AllocSoa + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}

impl<T, U> AsRef<[U]> for Drain<'_, T>
where
    T: SoaOwned + AllocSoa + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Into<&'a [U]>,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Debug for Drain<'_, T>
where
    T: SoaOwned + AllocSoa + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("Drain").field(&slices).finish()
    }
}

impl<T> Iterator for Drain<'_, T>
where
    T: AllocSoa + SoaRead,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { iter, .. } = self;

        iter.as_raw_iter_mut().next().map(|src| {
            let context = iter.context();
            unsafe { T::read(context, src) }
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { iter, .. } = self;
        iter.as_raw_iter().size_hint()
    }
}

impl<T> DoubleEndedIterator for Drain<'_, T>
where
    T: AllocSoa + SoaRead,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { iter, .. } = self;

        iter.as_raw_iter_mut().next_back().map(|src| {
            let context = iter.context();
            unsafe { T::read(context, src) }
        })
    }
}

impl<T> ExactSizeIterator for Drain<'_, T>
where
    T: AllocSoa + SoaRead,
{
    #[inline]
    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<T> FusedIterator for Drain<'_, T> where T: AllocSoa + SoaRead {}

impl<T> Drop for Drain<'_, T>
where
    T: AllocSoa + ?Sized,
{
    fn drop(&mut self) {
        /// Moves back the un-`Drain`ed elements to restore the original `Vec`.
        struct DropGuard<'r, 'a, T>(&'r mut Drain<'a, T>)
        where
            T: AllocSoa + ?Sized;

        impl<T> Drop for DropGuard<'_, '_, T>
        where
            T: AllocSoa + ?Sized,
        {
            fn drop(&mut self) {
                let Self(drain) = self;
                let Drain {
                    tail_start,
                    tail_len,
                    mut vec,
                    ..
                } = **drain;

                if tail_len == 0 {
                    return;
                }

                // memory-move back untouched tail, update to new length
                let vec = unsafe { vec.as_mut() };
                let start = vec.len();
                let tail = tail_start;
                if tail != start {
                    let (context, ptrs) = vec.as_mut_ptrs_with_context();

                    let src = context.ptrs_cast_const(ptrs.clone());
                    let src = unsafe { context.ptrs_add(src, tail) };
                    let dst = unsafe { context.ptrs_add_mut(ptrs, start) };
                    unsafe {
                        context.ptrs_copy(src, dst, tail_len);
                    }
                }
                unsafe {
                    vec.set_len(start + tail_len);
                }
            }
        }

        let Self {
            ref iter,
            tail_len,
            mut vec,
            ..
        } = *self;
        let drop_len = iter.len();

        let context = unsafe { vec.as_ref() }.context();
        if is_zst::<T>(context) {
            // ZSTs have no identity, so we don't need to move them around, we only need to drop the correct amount.
            // this can be achieved by manipulating the Vec length instead of moving values out from `iter`.
            let vec = unsafe { vec.as_mut() };
            let old_len = vec.len();
            unsafe {
                vec.set_len(old_len + drop_len + tail_len);
                vec.truncate(old_len + tail_len);
            }
            return;
        }

        // ensure elements are moved back into their appropriate places, even when drop_in_place panics
        let guard = DropGuard(self);
        if drop_len == 0 {
            return;
        }

        // as_ptrs() must only be called when iter.len() is > 0 because
        // it also gets touched by vec::Splice which may turn it into a dangling pointer
        // which would make it and the vec pointer point to different allocations which would
        // lead to invalid pointer arithmetic below.
        let drop_ptrs = guard.0.iter.as_ptrs();

        // drop_ptrs comes from an Iter which only gives us slices but for drop_in_place
        // a pointer with mutable provenance is necessary. Therefore we must reconstruct
        // it from the original vec but also avoid creating a &mut to the front since that could
        // invalidate raw pointers to it which some unsafe code might rely on.
        let (context, vec_ptrs) = unsafe { vec.as_mut() }.as_mut_ptrs_with_context();
        let origin = context.ptrs_cast_const(vec_ptrs.clone());

        unsafe {
            let drop_offset = context.ptrs_offset_from(drop_ptrs, origin);
            let drop_offset = usize::try_from(drop_offset).unwrap_unchecked();

            let ptrs = context.ptrs_add_mut(vec_ptrs, drop_offset);
            let to_drop = context.mut_slice_ptrs_from_raw_parts(ptrs, drop_len);
            context.slices_drop_in_place(to_drop);
        }
    }
}
