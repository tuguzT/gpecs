use core::ops;

use crate::traits::Soa;

#[allow(clippy::missing_safety_doc)]
pub unsafe trait SoaSliceIndex<T>: private_slice_index::Sealed
where
    T: Soa,
{
    type Refs<'a>
    where
        T: 'a;

    type RefsMut<'a>
    where
        T: 'a;

    fn get<'a>(self, slices: T::Slices<'a>) -> Option<Self::Refs<'a>>;

    fn get_mut<'a>(self, slices: T::SlicesMut<'a>) -> Option<Self::RefsMut<'a>>;

    fn index<'a>(self, slices: T::Slices<'a>) -> Self::Refs<'a>;

    fn index_mut<'a>(self, slices: T::SlicesMut<'a>) -> Self::RefsMut<'a>;

    type Ptrs;

    type MutPtrs;

    unsafe fn get_unchecked(self, slices: T::SlicePtrs) -> Self::Ptrs;

    unsafe fn get_unchecked_mut(self, slices: T::SliceMutPtrs) -> Self::MutPtrs;
}

unsafe impl<T> SoaSliceIndex<T> for usize
where
    T: Soa,
{
    type Refs<'a>
        = T::Refs<'a>
    where
        T: 'a;

    type RefsMut<'a>
        = T::RefsMut<'a>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slices: T::Slices<'a>) -> Option<Self::Refs<'a>> {
        let slices = T::slice_refs_as_slice_ptrs(slices);
        let len = T::slice_ptrs_len(slices);
        if self >= len {
            return None;
        }

        unsafe {
            let ptrs = SoaSliceIndex::<T>::get_unchecked(self, slices);
            let refs = T::ptrs_to_refs(ptrs);
            Some(refs)
        }
    }

    #[inline]
    fn get_mut<'a>(self, slices: T::SlicesMut<'a>) -> Option<Self::RefsMut<'a>> {
        let slices = T::mut_slice_refs_as_slice_ptrs(slices);
        let len = T::slice_ptrs_len_mut(slices);
        if self >= len {
            return None;
        }

        unsafe {
            let ptrs = SoaSliceIndex::<T>::get_unchecked_mut(self, slices);
            let refs = T::ptrs_to_refs_mut(ptrs);
            Some(refs)
        }
    }

    #[inline]
    fn index<'a>(self, slices: T::Slices<'a>) -> Self::Refs<'a> {
        let len = T::slices_len(&slices);
        match SoaSliceIndex::<T>::get(self, slices) {
            Some(value) => value,
            None => slice_index_usize_fail(len, self),
        }
    }

    #[inline]
    fn index_mut<'a>(self, slices: T::SlicesMut<'a>) -> Self::RefsMut<'a> {
        let len = T::slices_len_mut(&slices);
        match SoaSliceIndex::<T>::get_mut(self, slices) {
            Some(value) => value,
            None => slice_index_usize_fail(len, self),
        }
    }

    type Ptrs = T::Ptrs;

    type MutPtrs = T::MutPtrs;

    unsafe fn get_unchecked(self, slices: T::SlicePtrs) -> Self::Ptrs {
        let len = T::slice_ptrs_len(slices);
        debug_assert!(
            self < len,
            "slice::get_unchecked requires that the index is within the slice",
        );

        let ptrs = T::slice_ptrs_as_ptrs(slices);
        unsafe { T::ptrs_add(ptrs, self) }
    }

    unsafe fn get_unchecked_mut(self, slices: T::SliceMutPtrs) -> Self::MutPtrs {
        let len = T::slice_ptrs_len_mut(slices);
        debug_assert!(
            self < len,
            "slice::get_unchecked_mut requires that the index is within the slice",
        );

        let ptrs = T::mut_slice_ptrs_as_ptrs(slices);
        unsafe { T::ptrs_add_mut(ptrs, self) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::Range<usize>
where
    T: Soa,
{
    type Refs<'a>
        = T::Slices<'a>
    where
        T: 'a;

    type RefsMut<'a>
        = T::SlicesMut<'a>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slices: T::Slices<'a>) -> Option<Self::Refs<'a>> {
        let slices = T::slice_refs_as_slice_ptrs(slices);
        let len = T::slice_ptrs_len(slices);
        if self.start > self.end || self.end > len {
            return None;
        }

        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked(self, slices);
            let slices = T::slice_ptrs_to_slices(slices);
            Some(slices)
        }
    }

    #[inline]
    fn get_mut<'a>(self, slices: T::SlicesMut<'a>) -> Option<Self::RefsMut<'a>> {
        let slices = T::mut_slice_refs_as_slice_ptrs(slices);
        let len = T::slice_ptrs_len_mut(slices);
        if self.start > self.end || self.end > len {
            return None;
        }

        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked_mut(self, slices);
            let slices = T::slice_ptrs_to_slices_mut(slices);
            Some(slices)
        }
    }

    #[inline]
    fn index<'a>(self, slices: T::Slices<'a>) -> Self::Refs<'a> {
        let len = T::slices_len(&slices);
        if self.start > self.end {
            slice_index_order_fail(self.start, self.end);
        } else if self.end > len {
            slice_end_index_len_fail(self.end, len);
        }

        let slices = T::slice_refs_as_slice_ptrs(slices);
        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked(self, slices);
            T::slice_ptrs_to_slices(slices)
        }
    }

    #[inline]
    fn index_mut<'a>(self, slices: T::SlicesMut<'a>) -> Self::RefsMut<'a> {
        let len = T::slices_len_mut(&slices);
        if self.start > self.end {
            slice_index_order_fail(self.start, self.end);
        } else if self.end > len {
            slice_end_index_len_fail(self.end, len);
        }

        let slices = T::mut_slice_refs_as_slice_ptrs(slices);
        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked_mut(self, slices);
            T::slice_ptrs_to_slices_mut(slices)
        }
    }

    type Ptrs = T::SlicePtrs;

    type MutPtrs = T::SliceMutPtrs;

    unsafe fn get_unchecked(self, slices: T::SlicePtrs) -> Self::Ptrs {
        let len = T::slice_ptrs_len(slices);
        debug_assert!(
            self.end >= self.start && self.end <= len,
            "slice::get_unchecked requires that the range is within the slice",
        );

        let ptrs = T::slice_ptrs_as_ptrs(slices);
        unsafe {
            let ptrs = T::ptrs_add(ptrs, self.start);
            let new_len = self.end.unchecked_sub(self.start);
            T::slices_from_raw_parts(ptrs, new_len)
        }
    }

    unsafe fn get_unchecked_mut(self, slices: T::SliceMutPtrs) -> Self::MutPtrs {
        let len = T::slice_ptrs_len_mut(slices);
        debug_assert!(
            self.end >= self.start && self.end <= len,
            "slice::get_unchecked_mut requires that the range is within the slice",
        );

        let ptrs = T::mut_slice_ptrs_as_ptrs(slices);
        unsafe {
            let ptrs = T::ptrs_add_mut(ptrs, self.start);
            let new_len = self.end.unchecked_sub(self.start);
            T::slices_from_raw_parts_mut(ptrs, new_len)
        }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeTo<usize>
where
    T: Soa,
{
    type Refs<'a>
        = T::Slices<'a>
    where
        T: 'a;

    type RefsMut<'a>
        = T::SlicesMut<'a>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slices: T::Slices<'a>) -> Option<Self::Refs<'a>> {
        SoaSliceIndex::<T>::get(0..self.end, slices)
    }

    #[inline]
    fn get_mut<'a>(self, slices: T::SlicesMut<'a>) -> Option<Self::RefsMut<'a>> {
        SoaSliceIndex::<T>::get_mut(0..self.end, slices)
    }

    #[inline]
    fn index<'a>(self, slices: T::Slices<'a>) -> Self::Refs<'a> {
        SoaSliceIndex::<T>::index(0..self.end, slices)
    }

    #[inline]
    fn index_mut<'a>(self, slices: T::SlicesMut<'a>) -> Self::RefsMut<'a> {
        SoaSliceIndex::<T>::index_mut(0..self.end, slices)
    }

    type Ptrs = T::SlicePtrs;

    type MutPtrs = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slices: T::SlicePtrs) -> Self::Ptrs {
        unsafe { SoaSliceIndex::<T>::get_unchecked(0..self.end, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slices: T::SliceMutPtrs) -> Self::MutPtrs {
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(0..self.end, slices) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeFrom<usize>
where
    T: Soa,
{
    type Refs<'a>
        = T::Slices<'a>
    where
        T: 'a;

    type RefsMut<'a>
        = T::SlicesMut<'a>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slices: T::Slices<'a>) -> Option<Self::Refs<'a>> {
        let len = T::slices_len(&slices);
        SoaSliceIndex::<T>::get(self.start..len, slices)
    }

    #[inline]
    fn get_mut<'a>(self, slices: T::SlicesMut<'a>) -> Option<Self::RefsMut<'a>> {
        let len = T::slices_len_mut(&slices);
        SoaSliceIndex::<T>::get_mut(self.start..len, slices)
    }

    #[inline]
    fn index<'a>(self, slices: T::Slices<'a>) -> Self::Refs<'a> {
        let len = T::slices_len(&slices);
        if self.start > len {
            slice_start_index_len_fail(self.start, len);
        }

        let slices = T::slice_refs_as_slice_ptrs(slices);
        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked(self, slices);
            T::slice_ptrs_to_slices(slices)
        }
    }

    #[inline]
    fn index_mut<'a>(self, slices: T::SlicesMut<'a>) -> Self::RefsMut<'a> {
        let len = T::slices_len_mut(&slices);
        if self.start > len {
            slice_start_index_len_fail(self.start, len);
        }

        let slices = T::mut_slice_refs_as_slice_ptrs(slices);
        unsafe {
            let slices = SoaSliceIndex::<T>::get_unchecked_mut(self, slices);
            T::slice_ptrs_to_slices_mut(slices)
        }
    }

    type Ptrs = T::SlicePtrs;

    type MutPtrs = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slices: T::SlicePtrs) -> Self::Ptrs {
        let len = T::slice_ptrs_len(slices);
        unsafe { SoaSliceIndex::<T>::get_unchecked(self.start..len, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slices: T::SliceMutPtrs) -> Self::MutPtrs {
        let len = T::slice_ptrs_len_mut(slices);
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(self.start..len, slices) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeFull
where
    T: Soa,
{
    type Refs<'a>
        = T::Slices<'a>
    where
        T: 'a;

    type RefsMut<'a>
        = T::SlicesMut<'a>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slices: T::Slices<'a>) -> Option<Self::Refs<'a>> {
        Some(slices)
    }

    #[inline]
    fn get_mut<'a>(self, slices: T::SlicesMut<'a>) -> Option<Self::RefsMut<'a>> {
        Some(slices)
    }

    #[inline]
    fn index<'a>(self, slices: T::Slices<'a>) -> Self::Refs<'a> {
        slices
    }

    #[inline]
    fn index_mut<'a>(self, slices: T::SlicesMut<'a>) -> Self::RefsMut<'a> {
        slices
    }

    type Ptrs = T::SlicePtrs;

    type MutPtrs = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slices: T::SlicePtrs) -> Self::Ptrs {
        slices
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slices: T::SliceMutPtrs) -> Self::MutPtrs {
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
    type Refs<'a>
        = T::Slices<'a>
    where
        T: 'a;

    type RefsMut<'a>
        = T::SlicesMut<'a>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slices: T::Slices<'a>) -> Option<Self::Refs<'a>> {
        if *self.end() == usize::MAX {
            return None;
        }
        SoaSliceIndex::<T>::get(range_into_slice_range(self), slices)
    }

    #[inline]
    fn get_mut<'a>(self, slices: T::SlicesMut<'a>) -> Option<Self::RefsMut<'a>> {
        if *self.end() == usize::MAX {
            return None;
        }
        SoaSliceIndex::<T>::get_mut(range_into_slice_range(self), slices)
    }

    #[inline]
    fn index<'a>(self, slices: T::Slices<'a>) -> Self::Refs<'a> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        SoaSliceIndex::<T>::index(range_into_slice_range(self), slices)
    }

    #[inline]
    fn index_mut<'a>(self, slices: T::SlicesMut<'a>) -> Self::RefsMut<'a> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        SoaSliceIndex::<T>::index_mut(range_into_slice_range(self), slices)
    }

    type Ptrs = T::SlicePtrs;

    type MutPtrs = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slices: T::SlicePtrs) -> Self::Ptrs {
        unsafe { SoaSliceIndex::<T>::get_unchecked(range_into_slice_range(self), slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slices: T::SliceMutPtrs) -> Self::MutPtrs {
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(range_into_slice_range(self), slices) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeToInclusive<usize>
where
    T: Soa,
{
    type Refs<'a>
        = T::Slices<'a>
    where
        T: 'a;

    type RefsMut<'a>
        = T::SlicesMut<'a>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slices: T::Slices<'a>) -> Option<Self::Refs<'a>> {
        SoaSliceIndex::<T>::get(0..=self.end, slices)
    }

    #[inline]
    fn get_mut<'a>(self, slices: T::SlicesMut<'a>) -> Option<Self::RefsMut<'a>> {
        SoaSliceIndex::<T>::get_mut(0..=self.end, slices)
    }

    #[inline]
    fn index<'a>(self, slices: T::Slices<'a>) -> Self::Refs<'a> {
        SoaSliceIndex::<T>::index(0..=self.end, slices)
    }

    #[inline]
    fn index_mut<'a>(self, slices: T::SlicesMut<'a>) -> Self::RefsMut<'a> {
        SoaSliceIndex::<T>::index_mut(0..=self.end, slices)
    }

    type Ptrs = T::SlicePtrs;

    type MutPtrs = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slices: T::SlicePtrs) -> Self::Ptrs {
        unsafe { SoaSliceIndex::<T>::get_unchecked(0..=self.end, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slices: T::SliceMutPtrs) -> Self::MutPtrs {
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(0..=self.end, slices) }
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
    type Refs<'a>
        = T::Slices<'a>
    where
        T: 'a;

    type RefsMut<'a>
        = T::SlicesMut<'a>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slices: T::Slices<'a>) -> Option<Self::Refs<'a>> {
        let len = T::slices_len(&slices);
        SoaSliceIndex::<T>::get(into_range(len, self)?, slices)
    }

    #[inline]
    fn get_mut<'a>(self, slices: T::SlicesMut<'a>) -> Option<Self::RefsMut<'a>> {
        let len = T::slices_len_mut(&slices);
        SoaSliceIndex::<T>::get_mut(into_range(len, self)?, slices)
    }

    #[inline]
    fn index<'a>(self, slices: T::Slices<'a>) -> Self::Refs<'a> {
        let len = T::slices_len(&slices);
        SoaSliceIndex::<T>::index(into_slice_range(len, self), slices)
    }

    #[inline]
    fn index_mut<'a>(self, slices: T::SlicesMut<'a>) -> Self::RefsMut<'a> {
        let len = T::slices_len_mut(&slices);
        SoaSliceIndex::<T>::index_mut(into_slice_range(len, self), slices)
    }

    type Ptrs = T::SlicePtrs;

    type MutPtrs = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slices: T::SlicePtrs) -> Self::Ptrs {
        let len = T::slice_ptrs_len(slices);
        unsafe { SoaSliceIndex::<T>::get_unchecked(into_range_unchecked(len, self), slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slices: T::SliceMutPtrs) -> Self::MutPtrs {
        let len = T::slice_ptrs_len_mut(slices);
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(into_range_unchecked(len, self), slices) }
    }
}

#[cold]
#[inline(never)]
#[track_caller]
pub(super) fn slice_index_usize_fail(len: usize, index: usize) -> ! {
    panic!("index out of bounds: the len is {len} but the index is {index}")
}

#[cold]
#[inline(never)]
#[track_caller]
pub(super) fn slice_index_order_fail(index: usize, end: usize) -> ! {
    panic!("slice index starts at {index} but ends at {end}");
}

#[inline]
#[track_caller]
fn slice_start_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range start index {index} out of range for slice of length {len}");
}

#[cold]
#[inline(never)]
#[track_caller]
pub(super) fn slice_end_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range end index {index} out of range for slice of length {len}");
}

#[cold]
#[inline(never)]
#[track_caller]
pub(super) const fn slice_end_index_overflow_fail() -> ! {
    panic!("attempted to index slice up to maximum usize");
}

#[cold]
#[inline(never)]
#[track_caller]
pub(super) const fn slice_start_index_overflow_fail() -> ! {
    panic!("attempted to index slice from after maximum usize");
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
