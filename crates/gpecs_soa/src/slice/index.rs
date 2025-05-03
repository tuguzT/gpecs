use core::ops;

use crate::traits::Soa;

use super::{
    slice_end_index_len_fail, slice_end_index_overflow_fail, slice_index_order_fail,
    slice_index_usize_fail, slice_start_index_len_fail, slice_start_index_overflow_fail,
};

#[allow(clippy::missing_safety_doc)]
pub unsafe trait SoaSliceIndex<T>: private_slice_index::Sealed
where
    T: Soa,
{
    type Refs<'context, 'a>
    where
        T: 'a;

    type RefsMut<'context, 'a>
    where
        T: 'a;

    fn get<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Option<Self::Refs<'context, 'a>>;

    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>>;

    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a>;

    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a>;

    type Ptrs<'context>;

    type MutPtrs<'context>;

    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: T::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context>;

    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context>;
}

unsafe impl<T> SoaSliceIndex<T> for usize
where
    T: Soa,
{
    type Refs<'context, 'a>
        = T::Refs<'context, 'a>
    where
        T: 'a;

    type RefsMut<'context, 'a>
        = T::RefsMut<'context, 'a>
    where
        T: 'a;

    #[inline]
    fn get<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Option<Self::Refs<'context, 'a>> {
        let slices = T::slices_as_slice_ptrs(context, slices);
        let len = T::slice_ptrs_len(context, &slices);
        if self >= len {
            return None;
        }

        unsafe {
            let ptrs = SoaSliceIndex::<T>::get_unchecked(self, context, slices);
            let refs = T::ptrs_to_refs(context, ptrs);
            Some(refs)
        }
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        let len = T::slice_mut_ptrs_len(context, &slices);
        if self >= len {
            return None;
        }

        unsafe {
            let ptrs = SoaSliceIndex::<T>::get_unchecked_mut(self, context, slices);
            let refs = T::ptrs_to_refs_mut(context, ptrs);
            Some(refs)
        }
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let len = T::slices_len(context, &slices);
        match SoaSliceIndex::<T>::get(self, context, slices) {
            Some(value) => value,
            None => slice_index_usize_fail(len, self),
        }
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        let len = T::slices_mut_len(context, &slices);
        match SoaSliceIndex::<T>::get_mut(self, context, slices) {
            Some(value) => value,
            None => slice_index_usize_fail(len, self),
        }
    }

    type Ptrs<'context> = T::Ptrs<'context>;

    type MutPtrs<'context> = T::MutPtrs<'context>;

    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: T::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let len = T::slice_ptrs_len(context, &slices);
        debug_assert!(
            self < len,
            "slice::get_unchecked requires that the index is within the slice",
        );

        let ptrs = T::slice_ptrs_as_ptrs(context, slices);
        unsafe { T::ptrs_add(context, ptrs, self) }
    }

    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let len = T::slice_mut_ptrs_len(context, &slices);
        debug_assert!(
            self < len,
            "slice::get_unchecked_mut requires that the index is within the slice",
        );

        let ptrs = T::slice_mut_ptrs_as_ptrs(context, slices);
        unsafe { T::ptrs_add_mut(context, ptrs, self) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::Range<usize>
where
    T: Soa,
{
    type Refs<'context, 'a>
        = T::Slices<'context, 'a>
    where
        T: 'a;

    type RefsMut<'context, 'a>
        = T::SlicesMut<'context, 'a>
    where
        T: 'a;

    #[inline]
    fn get<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Option<Self::Refs<'context, 'a>> {
        let slices = T::slices_as_slice_ptrs(context, slices);
        let len = T::slice_ptrs_len(context, &slices);
        if self.start > self.end || self.end > len {
            return None;
        }

        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked(self, context, slices);
            let slices = T::slice_ptrs_to_slices(context, slices);
            Some(slices)
        }
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        let len = T::slice_mut_ptrs_len(context, &slices);
        if self.start > self.end || self.end > len {
            return None;
        }

        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked_mut(self, context, slices);
            let slices = T::slice_mut_ptrs_to_slices(context, slices);
            Some(slices)
        }
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let len = T::slices_len(context, &slices);
        if self.start > self.end {
            slice_index_order_fail(self.start, self.end);
        } else if self.end > len {
            slice_end_index_len_fail(self.end, len);
        }

        let slices = T::slices_as_slice_ptrs(context, slices);
        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked(self, context, slices);
            T::slice_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        let len = T::slices_mut_len(context, &slices);
        if self.start > self.end {
            slice_index_order_fail(self.start, self.end);
        } else if self.end > len {
            slice_end_index_len_fail(self.end, len);
        }

        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked_mut(self, context, slices);
            T::slice_mut_ptrs_to_slices(context, slices)
        }
    }

    type Ptrs<'context> = T::SlicePtrs<'context>;

    type MutPtrs<'context> = T::SliceMutPtrs<'context>;

    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: T::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let len = T::slice_ptrs_len(context, &slices);
        debug_assert!(
            self.end >= self.start && self.end <= len,
            "slice::get_unchecked requires that the range is within the slice",
        );

        let ptrs = T::slice_ptrs_as_ptrs(context, slices);
        unsafe {
            let ptrs = T::ptrs_add(context, ptrs, self.start);
            let new_len = self.end.unchecked_sub(self.start);
            T::slices_from_raw_parts(context, ptrs, new_len)
        }
    }

    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let len = T::slice_mut_ptrs_len(context, &slices);
        debug_assert!(
            self.end >= self.start && self.end <= len,
            "slice::get_unchecked_mut requires that the range is within the slice",
        );

        let ptrs = T::slice_mut_ptrs_as_ptrs(context, slices);
        unsafe {
            let ptrs = T::ptrs_add_mut(context, ptrs, self.start);
            let new_len = self.end.unchecked_sub(self.start);
            T::slices_from_raw_parts_mut(context, ptrs, new_len)
        }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeTo<usize>
where
    T: Soa,
{
    type Refs<'context, 'a>
        = T::Slices<'context, 'a>
    where
        T: 'a;

    type RefsMut<'context, 'a>
        = T::SlicesMut<'context, 'a>
    where
        T: 'a;

    #[inline]
    fn get<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Option<Self::Refs<'context, 'a>> {
        SoaSliceIndex::<T>::get(0..self.end, context, slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        SoaSliceIndex::<T>::get_mut(0..self.end, context, slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        SoaSliceIndex::<T>::index(0..self.end, context, slices)
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        SoaSliceIndex::<T>::index_mut(0..self.end, context, slices)
    }

    type Ptrs<'context> = T::SlicePtrs<'context>;

    type MutPtrs<'context> = T::SliceMutPtrs<'context>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: T::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        unsafe { SoaSliceIndex::<T>::get_unchecked(0..self.end, context, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(0..self.end, context, slices) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeFrom<usize>
where
    T: Soa,
{
    type Refs<'context, 'a>
        = T::Slices<'context, 'a>
    where
        T: 'a;

    type RefsMut<'context, 'a>
        = T::SlicesMut<'context, 'a>
    where
        T: 'a;

    #[inline]
    fn get<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Option<Self::Refs<'context, 'a>> {
        let len = T::slices_len(context, &slices);
        SoaSliceIndex::<T>::get(self.start..len, context, slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let len = T::slices_mut_len(context, &slices);
        SoaSliceIndex::<T>::get_mut(self.start..len, context, slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let len = T::slices_len(context, &slices);
        if self.start > len {
            slice_start_index_len_fail(self.start, len);
        }

        let slices = T::slices_as_slice_ptrs(context, slices);
        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked(self, context, slices);
            T::slice_ptrs_to_slices(context, slices)
        }
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        let len = T::slices_mut_len(context, &slices);
        if self.start > len {
            slice_start_index_len_fail(self.start, len);
        }

        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked_mut(self, context, slices);
            T::slice_mut_ptrs_to_slices(context, slices)
        }
    }

    type Ptrs<'context> = T::SlicePtrs<'context>;

    type MutPtrs<'context> = T::SliceMutPtrs<'context>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: T::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let len = T::slice_ptrs_len(context, &slices);
        unsafe { SoaSliceIndex::<T>::get_unchecked(self.start..len, context, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let len = T::slice_mut_ptrs_len(context, &slices);
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(self.start..len, context, slices) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeFull
where
    T: Soa,
{
    type Refs<'context, 'a>
        = T::Slices<'context, 'a>
    where
        T: 'a;

    type RefsMut<'context, 'a>
        = T::SlicesMut<'context, 'a>
    where
        T: 'a;

    #[inline]
    fn get<'context, 'a>(
        self,
        _context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Option<Self::Refs<'context, 'a>> {
        Some(slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        _context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        Some(slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        _context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        slices
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        _context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        slices
    }

    type Ptrs<'context> = T::SlicePtrs<'context>;

    type MutPtrs<'context> = T::SliceMutPtrs<'context>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        _context: &'context T::Context,
        slices: T::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        slices
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        _context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        slices
    }
}

/// Based on implementation of 2 methods:
/// - [`core::ops::RangeInclusive::into_slice_range()`]
/// - [`core::ops::RangeInclusive::is_empty()`] which replaces access to [`core::ops::RangeInclusive::exhausted`] private field
#[inline]
fn range_into_slice_range(range: ops::RangeInclusive<usize>) -> ops::Range<usize> {
    let exclusive_end = range.end() + 1;

    let exhausted = range.is_empty();
    let start = if exhausted {
        exclusive_end
    } else {
        *range.start()
    };

    start..exclusive_end
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeInclusive<usize>
where
    T: Soa,
{
    type Refs<'context, 'a>
        = T::Slices<'context, 'a>
    where
        T: 'a;

    type RefsMut<'context, 'a>
        = T::SlicesMut<'context, 'a>
    where
        T: 'a;

    #[inline]
    fn get<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Option<Self::Refs<'context, 'a>> {
        if *self.end() == usize::MAX {
            return None;
        }
        SoaSliceIndex::<T>::get(range_into_slice_range(self), context, slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        if *self.end() == usize::MAX {
            return None;
        }
        SoaSliceIndex::<T>::get_mut(range_into_slice_range(self), context, slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        SoaSliceIndex::<T>::index(range_into_slice_range(self), context, slices)
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        SoaSliceIndex::<T>::index_mut(range_into_slice_range(self), context, slices)
    }

    type Ptrs<'context> = T::SlicePtrs<'context>;

    type MutPtrs<'context> = T::SliceMutPtrs<'context>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: T::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        unsafe { SoaSliceIndex::<T>::get_unchecked(range_into_slice_range(self), context, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        unsafe {
            SoaSliceIndex::<T>::get_unchecked_mut(range_into_slice_range(self), context, slices)
        }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeToInclusive<usize>
where
    T: Soa,
{
    type Refs<'context, 'a>
        = T::Slices<'context, 'a>
    where
        T: 'a;

    type RefsMut<'context, 'a>
        = T::SlicesMut<'context, 'a>
    where
        T: 'a;

    #[inline]
    fn get<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Option<Self::Refs<'context, 'a>> {
        SoaSliceIndex::<T>::get(0..=self.end, context, slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        SoaSliceIndex::<T>::get_mut(0..=self.end, context, slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        SoaSliceIndex::<T>::index(0..=self.end, context, slices)
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        SoaSliceIndex::<T>::index_mut(0..=self.end, context, slices)
    }

    type Ptrs<'context> = T::SlicePtrs<'context>;

    type MutPtrs<'context> = T::SliceMutPtrs<'context>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: T::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        unsafe { SoaSliceIndex::<T>::get_unchecked(0..=self.end, context, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(0..=self.end, context, slices) }
    }
}

/// Copy of private [`core::slice::index::into_range_unchecked()`] function.
fn into_range_unchecked(
    len: usize,
    (start, end): (ops::Bound<usize>, ops::Bound<usize>),
) -> ops::Range<usize> {
    use ops::Bound;
    let start = match start {
        Bound::Included(i) => i,
        Bound::Excluded(i) => i + 1,
        Bound::Unbounded => 0,
    };
    let end = match end {
        Bound::Included(i) => i + 1,
        Bound::Excluded(i) => i,
        Bound::Unbounded => len,
    };
    start..end
}

/// Copy of private [`core::slice::index::into_range()`] function.
fn into_range(
    len: usize,
    (start, end): (ops::Bound<usize>, ops::Bound<usize>),
) -> Option<ops::Range<usize>> {
    use ops::Bound;
    let start = match start {
        Bound::Included(start) => start,
        Bound::Excluded(start) => start.checked_add(1)?,
        Bound::Unbounded => 0,
    };

    let end = match end {
        Bound::Included(end) => end.checked_add(1)?,
        Bound::Excluded(end) => end,
        Bound::Unbounded => len,
    };

    // Don't bother with checking `start < end` and `end <= len`
    // since these checks are handled by `Range` impls

    Some(start..end)
}

/// Copy of private [`core::slice::index::into_slice_range()`] function.
fn into_slice_range(
    len: usize,
    (start, end): (ops::Bound<usize>, ops::Bound<usize>),
) -> ops::Range<usize> {
    use ops::Bound;
    let start = match start {
        Bound::Included(start) => start,
        Bound::Excluded(start) => start
            .checked_add(1)
            .unwrap_or_else(|| slice_start_index_overflow_fail()),
        Bound::Unbounded => 0,
    };

    let end = match end {
        Bound::Included(end) => end
            .checked_add(1)
            .unwrap_or_else(|| slice_end_index_overflow_fail()),
        Bound::Excluded(end) => end,
        Bound::Unbounded => len,
    };

    // Don't bother with checking `start < end` and `end <= len`
    // since these checks are handled by `Range` impls

    start..end
}

unsafe impl<T> SoaSliceIndex<T> for (ops::Bound<usize>, ops::Bound<usize>)
where
    T: Soa,
{
    type Refs<'context, 'a>
        = T::Slices<'context, 'a>
    where
        T: 'a;

    type RefsMut<'context, 'a>
        = T::SlicesMut<'context, 'a>
    where
        T: 'a;

    #[inline]
    fn get<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Option<Self::Refs<'context, 'a>> {
        let len = T::slices_len(context, &slices);
        SoaSliceIndex::<T>::get(into_range(len, self)?, context, slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let len = T::slices_mut_len(context, &slices);
        SoaSliceIndex::<T>::get_mut(into_range(len, self)?, context, slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let len = T::slices_len(context, &slices);
        SoaSliceIndex::<T>::index(into_slice_range(len, self), context, slices)
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        let len = T::slices_mut_len(context, &slices);
        SoaSliceIndex::<T>::index_mut(into_slice_range(len, self), context, slices)
    }

    type Ptrs<'context> = T::SlicePtrs<'context>;

    type MutPtrs<'context> = T::SliceMutPtrs<'context>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: T::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let len = T::slice_ptrs_len(context, &slices);
        unsafe {
            SoaSliceIndex::<T>::get_unchecked(into_range_unchecked(len, self), context, slices)
        }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let len = T::slice_mut_ptrs_len(context, &slices);
        unsafe {
            SoaSliceIndex::<T>::get_unchecked_mut(into_range_unchecked(len, self), context, slices)
        }
    }
}

mod private_slice_index {
    use core::ops;

    pub trait Sealed {}

    impl Sealed for usize {}

    impl Sealed for ops::Range<usize> {}

    impl Sealed for ops::RangeTo<usize> {}

    impl Sealed for ops::RangeFrom<usize> {}

    impl Sealed for ops::RangeFull {}

    impl Sealed for ops::RangeInclusive<usize> {}

    impl Sealed for ops::RangeToInclusive<usize> {}

    impl Sealed for (ops::Bound<usize>, ops::Bound<usize>) {}
}
