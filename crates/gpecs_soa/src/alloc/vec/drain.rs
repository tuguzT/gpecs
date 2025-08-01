use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::{Range, RangeBounds},
    ptr::{self, NonNull},
};

use crate::{
    ptr::is_zst,
    slice::{Iter, SoaSlices, range},
    traits::SoaRead,
};

use super::{Soa, SoaVec};

pub struct Drain<'a, T>
where
    T: Soa + ?Sized + 'a,
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
    T: Soa + ?Sized,
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

        unsafe {
            let mut vec = NonNull::from(vec);

            // index before setting length, otherwise range is invalid
            let context = vec.as_ref().context();
            let slices = vec.as_ref().slices().into_index(range);
            let slices = SoaSlices::new(context, slices);

            // set self.vec length's to start, to be safe in case Drain is leaked
            vec.as_mut().set_len(start);

            Self {
                tail_start: end,
                tail_len: len - end,
                iter: Iter::new(slices),
                vec,
            }
        }
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { iter, .. } = self;
        iter.context()
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let Self { iter, .. } = self;
        iter.as_slices()
    }
}

unsafe impl<T> Send for Drain<'_, T>
where
    T: Soa + ?Sized,
    T::Fields: Send,
    T::Context: Send,
{
}

unsafe impl<T> Sync for Drain<'_, T>
where
    T: Soa + ?Sized,
    T::Fields: Sync,
    T::Context: Sync,
{
}

impl<T, U> AsRef<[U]> for Drain<'_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T: Soa<Slices<'c, 'any> = &'any [U]> + 'any,
{
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> Debug for Drain<'_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("Drain").field(&slices).finish()
    }
}

impl<T> Iterator for Drain<'_, T>
where
    T: SoaRead,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { iter, .. } = self;

        let context = ptr::from_ref(iter.context());
        iter.next().map(|refs| unsafe {
            let context = &*context;
            T::read(context, T::refs_as_ptrs(context, refs))
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { iter, .. } = self;
        iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for Drain<'_, T>
where
    T: SoaRead,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { iter, .. } = self;

        let context = ptr::from_ref(iter.context());
        iter.next_back().map(|refs| unsafe {
            let context = &*context;
            T::read(context, T::refs_as_ptrs(context, refs))
        })
    }
}

impl<T> ExactSizeIterator for Drain<'_, T>
where
    T: SoaRead,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { iter, .. } = self;
        iter.len()
    }
}

impl<T> FusedIterator for Drain<'_, T> where T: SoaRead {}

impl<T> Drop for Drain<'_, T>
where
    T: Soa + ?Sized,
{
    fn drop(&mut self) {
        /// Moves back the un-`Drain`ed elements to restore the original `Vec`.
        struct DropGuard<'r, 'a, T>(&'r mut Drain<'a, T>)
        where
            T: Soa + ?Sized;

        impl<T> Drop for DropGuard<'_, '_, T>
        where
            T: Soa + ?Sized,
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
                unsafe {
                    let source_vec = vec.as_mut();
                    // memory-move back untouched tail, update to new length
                    let start = source_vec.len();
                    let tail = tail_start;
                    if tail != start {
                        let src = source_vec.as_ptrs();
                        let dst = source_vec.buffer.ptrs();
                        let context = source_vec.context();

                        let src = T::ptrs_add(context, src, tail);
                        let dst = T::ptrs_add_mut(context, dst, start);
                        T::ptrs_copy(context, src, dst, tail_len);
                    }
                    source_vec.set_len(start + tail_len);
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
            unsafe {
                let vec = vec.as_mut();
                let old_len = vec.len();
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

        unsafe {
            let vec_ptrs = vec.as_mut().as_mut_ptrs();
            let context = vec.as_ref().context();

            // as_ptrs() must only be called when iter.len() is > 0 because
            // it also gets touched by vec::Splice which may turn it into a dangling pointer
            // which would make it and the vec pointer point to different allocations which would
            // lead to invalid pointer arithmetic below.
            let slices = guard.0.iter.as_slices();
            let drop_ptrs = T::slices_as_ptrs(context, slices);

            // drop_ptrs comes from an Iter which only gives us slices but for drop_in_place
            // a pointer with mutable provenance is necessary. Therefore we must reconstruct
            // it from the original vec but also avoid creating a &mut to the front since that could
            // invalidate raw pointers to it which some unsafe code might rely on.
            let origin = T::ptrs_cast_const(context, vec_ptrs.clone());

            let drop_offset = T::ptrs_offset_from(context, drop_ptrs, origin);
            let drop_offset = usize::try_from(drop_offset).unwrap_unchecked();

            let ptrs = T::ptrs_add_mut(context, vec_ptrs, drop_offset);
            let to_drop = T::slices_from_raw_parts_mut(context, ptrs, drop_len);
            T::slices_drop_in_place(context, to_drop);
        }
    }
}
