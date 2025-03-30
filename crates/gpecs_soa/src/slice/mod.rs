use core::ops;

use crate::traits::Soa;

pub use self::{
    dst::{from_raw_parts, from_raw_parts_mut, SoaSlice},
    index::SoaSliceIndex,
    iter::{Iter, IterMut},
    slices::{SoaSlices, SoaSlicesMut},
};

mod dst;
mod index;
mod iter;
mod slices;

pub(crate) trait IndexHelper<'a, T>
where
    Self: SoaSliceIndex<T, Refs<'a> = &'a Self::Output>,
    T: Soa + 'a,
{
    type Output: ?Sized + 'a;
}

impl<'a, T, I, U> IndexHelper<'a, T> for I
where
    U: ?Sized + 'a,
    T: Soa + 'a,
    I: SoaSliceIndex<T, Refs<'a> = &'a U>,
{
    type Output = U;
}

pub(crate) trait IndexHelperMut<'a, T>
where
    Self: IndexHelper<'a, T> + SoaSliceIndex<T, RefsMut<'a> = &'a mut Self::Output>,
    T: Soa + 'a,
{
}

impl<'a, T, I, U> IndexHelperMut<'a, T> for I
where
    U: ?Sized + 'a,
    T: Soa + 'a,
    I: IndexHelper<'a, T, Output = U> + SoaSliceIndex<T, RefsMut<'a> = &'a mut U>,
{
}

/// Just a copy of unstable [`core::slice::range`]
#[track_caller]
#[must_use]
pub(crate) fn slice_range<R>(range: R, bounds: ops::RangeTo<usize>) -> ops::Range<usize>
where
    R: ops::RangeBounds<usize>,
{
    let len = bounds.end;

    let start = match range.start_bound() {
        ops::Bound::Included(&start) => start,
        ops::Bound::Excluded(start) => start
            .checked_add(1)
            .unwrap_or_else(|| slice_start_index_overflow_fail()),
        ops::Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        ops::Bound::Included(end) => end
            .checked_add(1)
            .unwrap_or_else(|| slice_end_index_overflow_fail()),
        ops::Bound::Excluded(&end) => end,
        ops::Bound::Unbounded => len,
    };

    if start > end {
        slice_index_order_fail(start, end);
    }
    if end > len {
        slice_end_index_len_fail(end, len);
    }

    ops::Range { start, end }
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) fn slice_index_usize_fail(len: usize, index: usize) -> ! {
    panic!("index out of bounds: the len is {len} but the index is {index}")
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) fn slice_index_order_fail(index: usize, end: usize) -> ! {
    panic!("slice index starts at {index} but ends at {end}");
}

#[inline]
#[track_caller]
pub(crate) fn slice_start_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range start index {index} out of range for slice of length {len}");
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) fn slice_end_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range end index {index} out of range for slice of length {len}");
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) const fn slice_end_index_overflow_fail() -> ! {
    panic!("attempted to index slice up to maximum usize");
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) const fn slice_start_index_overflow_fail() -> ! {
    panic!("attempted to index slice from after maximum usize");
}
