use core::ops;

use crate::{
    ptr::{ptrs, SoaSlicePtr, SoaSlicePtrMut},
    traits::Soa,
};

use super::SoaSlice;

#[allow(clippy::missing_safety_doc)]
pub unsafe trait SoaSliceIndex<T>: private_slice_index::Sealed
where
    T: ?Sized,
{
    type Ref<'a>
    where
        T: 'a;

    type RefMut<'a>
    where
        T: 'a;

    fn get(self, slice: &T) -> Option<Self::Ref<'_>>;

    fn get_mut(self, slice: &mut T) -> Option<Self::RefMut<'_>>;

    fn index(self, slice: &T) -> Self::Ref<'_>;

    fn index_mut(self, slice: &mut T) -> Self::RefMut<'_>;

    type Ptr;

    type MutPtr;

    unsafe fn get_unchecked(self, slice: *const T) -> Self::Ptr;

    unsafe fn get_unchecked_mut(self, slice: *mut T) -> Self::MutPtr;
}

unsafe impl<T> SoaSliceIndex<SoaSlice<T>> for usize
where
    T: Soa,
{
    type Ref<'a>
        = T::Refs<'a>
    where
        SoaSlice<T>: 'a;

    type RefMut<'a>
        = T::RefsMut<'a>
    where
        SoaSlice<T>: 'a;

    #[inline]
    fn get(self, slice: &SoaSlice<T>) -> Option<Self::Ref<'_>> {
        if self >= slice.len() {
            return None;
        }

        unsafe {
            let ptrs = self.get_unchecked(slice);
            let refs = T::as_refs(ptrs);
            Some(refs)
        }
    }

    #[inline]
    fn get_mut(self, slice: &mut SoaSlice<T>) -> Option<Self::RefMut<'_>> {
        if self >= slice.len() {
            return None;
        }

        unsafe {
            let ptrs = self.get_unchecked_mut(slice);
            let refs = T::as_mut_refs(ptrs);
            Some(refs)
        }
    }

    #[inline]
    fn index(self, slice: &SoaSlice<T>) -> Self::Ref<'_> {
        match self.get(slice) {
            Some(value) => value,
            None => slice_index_usize_fail(slice.len(), self),
        }
    }

    #[inline]
    fn index_mut(self, slice: &mut SoaSlice<T>) -> Self::RefMut<'_> {
        let len = slice.len();
        match self.get_mut(slice) {
            Some(value) => value,
            None => slice_index_usize_fail(len, self),
        }
    }

    type Ptr = T::Ptrs;

    type MutPtr = T::MutPtrs;

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T>) -> Self::Ptr {
        unsafe {
            debug_assert!(
                self < (*slice).len(),
                "slice::get_unchecked requires that the index is within the slice",
            );
        }

        let ptr = slice.as_ptr().cast_mut();
        let capacity = slice.capacity();
        unsafe {
            let ptrs = ptrs::<T>(ptr, capacity).unwrap_unchecked();
            let ptrs = T::ptrs_add_mut(ptrs, self);
            T::ptrs_cast_const(ptrs)
        }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T>) -> Self::MutPtr {
        unsafe {
            debug_assert!(
                self < (*slice).len(),
                "slice::get_unchecked_mut requires that the index is within the slice",
            );
        }

        let ptr = slice.as_mut_ptr();
        let capacity = slice.capacity();
        unsafe {
            let ptrs = ptrs::<T>(ptr, capacity).unwrap_unchecked();
            T::ptrs_add_mut(ptrs, self)
        }
    }
}

unsafe impl<T> SoaSliceIndex<SoaSlice<T>> for ops::Range<usize>
where
    T: Soa,
{
    type Ref<'a>
        = T::Slices<'a>
    where
        SoaSlice<T>: 'a;

    type RefMut<'a>
        = T::SlicesMut<'a>
    where
        SoaSlice<T>: 'a;

    #[inline]
    fn get(self, slice: &SoaSlice<T>) -> Option<Self::Ref<'_>> {
        if self.start > self.end || self.end > slice.len() {
            return None;
        }

        unsafe {
            let slices = self.get_unchecked(slice);
            let slices = T::slices_as_refs(slices);
            Some(slices)
        }
    }

    #[inline]
    fn get_mut(self, slice: &mut SoaSlice<T>) -> Option<Self::RefMut<'_>> {
        if self.start > self.end || self.end > slice.len() {
            return None;
        }

        unsafe {
            let slices = self.get_unchecked_mut(slice);
            let slices = T::mut_slices_as_refs(slices);
            Some(slices)
        }
    }

    #[inline]
    fn index(self, slice: &SoaSlice<T>) -> Self::Ref<'_> {
        if self.start > self.end {
            slice_index_order_fail(self.start, self.end);
        } else if self.end > slice.len() {
            slice_end_index_len_fail(self.end, slice.len());
        }

        unsafe {
            let slices = self.get_unchecked(slice);
            T::slices_as_refs(slices)
        }
    }

    #[inline]
    fn index_mut(self, slice: &mut SoaSlice<T>) -> Self::RefMut<'_> {
        if self.start > self.end {
            slice_index_order_fail(self.start, self.end);
        } else if self.end > slice.len() {
            slice_end_index_len_fail(self.end, slice.len());
        }

        unsafe {
            let slices = self.get_unchecked_mut(slice);
            T::mut_slices_as_refs(slices)
        }
    }

    type Ptr = T::SlicePtrs;

    type MutPtr = T::SliceMutPtrs;

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T>) -> Self::Ptr {
        unsafe {
            debug_assert!(
                self.end >= self.start && self.end <= (*slice).len(),
                "slice::get_unchecked requires that the range is within the slice",
            );
        }

        let ptr = slice.as_ptr().cast_mut();
        let capacity = slice.capacity();
        unsafe {
            let ptrs = ptrs::<T>(ptr, capacity).unwrap_unchecked();
            let ptrs = T::ptrs_cast_const(ptrs);
            let ptrs = T::ptrs_add(ptrs, self.start);
            let new_len = self.end.unchecked_sub(self.start);
            T::slices_from_raw_parts(ptrs, new_len)
        }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T>) -> Self::MutPtr {
        unsafe {
            debug_assert!(
                self.end >= self.start && self.end <= (*slice).len(),
                "slice::get_unchecked_mut requires that the range is within the slice",
            );
        }

        let ptr = slice.as_mut_ptr();
        let capacity = slice.capacity();
        unsafe {
            let ptrs = ptrs::<T>(ptr, capacity).unwrap_unchecked();
            let ptrs = T::ptrs_add_mut(ptrs, self.start);
            let new_len = self.end.unchecked_sub(self.start);
            T::slices_from_raw_parts_mut(ptrs, new_len)
        }
    }
}

unsafe impl<T> SoaSliceIndex<SoaSlice<T>> for ops::RangeTo<usize>
where
    T: Soa,
{
    type Ref<'a>
        = T::Slices<'a>
    where
        SoaSlice<T>: 'a;

    type RefMut<'a>
        = T::SlicesMut<'a>
    where
        SoaSlice<T>: 'a;

    #[inline]
    fn get(self, slice: &SoaSlice<T>) -> Option<Self::Ref<'_>> {
        (0..self.end).get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut SoaSlice<T>) -> Option<Self::RefMut<'_>> {
        (0..self.end).get_mut(slice)
    }

    #[inline]
    fn index(self, slice: &SoaSlice<T>) -> Self::Ref<'_> {
        (0..self.end).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut SoaSlice<T>) -> Self::RefMut<'_> {
        (0..self.end).index_mut(slice)
    }

    type Ptr = T::SlicePtrs;

    type MutPtr = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slice: *const SoaSlice<T>) -> Self::Ptr {
        unsafe { (0..self.end).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T>) -> Self::MutPtr {
        unsafe { (0..self.end).get_unchecked_mut(slice) }
    }
}

unsafe impl<T> SoaSliceIndex<SoaSlice<T>> for ops::RangeFrom<usize>
where
    T: Soa,
{
    type Ref<'a>
        = T::Slices<'a>
    where
        SoaSlice<T>: 'a;

    type RefMut<'a>
        = T::SlicesMut<'a>
    where
        SoaSlice<T>: 'a;

    #[inline]
    fn get(self, slice: &SoaSlice<T>) -> Option<Self::Ref<'_>> {
        (self.start..slice.len()).get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut SoaSlice<T>) -> Option<Self::RefMut<'_>> {
        (self.start..slice.len()).get_mut(slice)
    }

    #[inline]
    fn index(self, slice: &SoaSlice<T>) -> Self::Ref<'_> {
        if self.start > slice.len() {
            slice_start_index_len_fail(self.start, slice.len());
        }

        unsafe {
            let slices = self.get_unchecked(slice);
            T::slices_as_refs(slices)
        }
    }

    #[inline]
    fn index_mut(self, slice: &mut SoaSlice<T>) -> Self::RefMut<'_> {
        if self.start > slice.len() {
            slice_start_index_len_fail(self.start, slice.len());
        }

        unsafe {
            let slices = self.get_unchecked_mut(slice);
            T::mut_slices_as_refs(slices)
        }
    }

    type Ptr = T::SlicePtrs;

    type MutPtr = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slice: *const SoaSlice<T>) -> Self::Ptr {
        unsafe { (self.start..slice.len()).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T>) -> Self::MutPtr {
        unsafe { (self.start..slice.len()).get_unchecked_mut(slice) }
    }
}

unsafe impl<T> SoaSliceIndex<SoaSlice<T>> for ops::RangeFull
where
    T: Soa,
{
    type Ref<'a>
        = T::Slices<'a>
    where
        SoaSlice<T>: 'a;

    type RefMut<'a>
        = T::SlicesMut<'a>
    where
        SoaSlice<T>: 'a;

    #[inline]
    fn get(self, slice: &SoaSlice<T>) -> Option<Self::Ref<'_>> {
        Some(slice.as_slices())
    }

    #[inline]
    fn get_mut(self, slice: &mut SoaSlice<T>) -> Option<Self::RefMut<'_>> {
        Some(slice.as_mut_slices())
    }

    #[inline]
    fn index(self, slice: &SoaSlice<T>) -> Self::Ref<'_> {
        slice.as_slices()
    }

    #[inline]
    fn index_mut(self, slice: &mut SoaSlice<T>) -> Self::RefMut<'_> {
        slice.as_mut_slices()
    }

    type Ptr = T::SlicePtrs;

    type MutPtr = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slice: *const SoaSlice<T>) -> Self::Ptr {
        let slices = unsafe { (*slice).as_slices() };
        T::slice_refs_as_slice_ptrs(slices)
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T>) -> Self::MutPtr {
        let slices = unsafe { (*slice).as_mut_slices() };
        T::mut_slice_refs_as_slice_ptrs(slices)
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

unsafe impl<T> SoaSliceIndex<SoaSlice<T>> for ops::RangeInclusive<usize>
where
    T: Soa,
{
    type Ref<'a>
        = T::Slices<'a>
    where
        SoaSlice<T>: 'a;

    type RefMut<'a>
        = T::SlicesMut<'a>
    where
        SoaSlice<T>: 'a;

    #[inline]
    fn get(self, slice: &SoaSlice<T>) -> Option<Self::Ref<'_>> {
        if *self.end() == usize::MAX {
            return None;
        }
        range_into_slice_range(self).get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut SoaSlice<T>) -> Option<Self::RefMut<'_>> {
        if *self.end() == usize::MAX {
            return None;
        }
        range_into_slice_range(self).get_mut(slice)
    }

    #[inline]
    fn index(self, slice: &SoaSlice<T>) -> Self::Ref<'_> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        range_into_slice_range(self).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut SoaSlice<T>) -> Self::RefMut<'_> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        range_into_slice_range(self).index_mut(slice)
    }

    type Ptr = T::SlicePtrs;

    type MutPtr = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slice: *const SoaSlice<T>) -> Self::Ptr {
        unsafe { range_into_slice_range(self).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T>) -> Self::MutPtr {
        unsafe { range_into_slice_range(self).get_unchecked_mut(slice) }
    }
}

unsafe impl<T> SoaSliceIndex<SoaSlice<T>> for ops::RangeToInclusive<usize>
where
    T: Soa,
{
    type Ref<'a>
        = T::Slices<'a>
    where
        SoaSlice<T>: 'a;

    type RefMut<'a>
        = T::SlicesMut<'a>
    where
        SoaSlice<T>: 'a;

    #[inline]
    fn get(self, slice: &SoaSlice<T>) -> Option<Self::Ref<'_>> {
        (0..=self.end).get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut SoaSlice<T>) -> Option<Self::RefMut<'_>> {
        (0..=self.end).get_mut(slice)
    }

    #[inline]
    fn index(self, slice: &SoaSlice<T>) -> Self::Ref<'_> {
        (0..=self.end).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut SoaSlice<T>) -> Self::RefMut<'_> {
        (0..=self.end).index_mut(slice)
    }

    type Ptr = T::SlicePtrs;

    type MutPtr = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slice: *const SoaSlice<T>) -> Self::Ptr {
        unsafe { (0..=self.end).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T>) -> Self::MutPtr {
        unsafe { (0..=self.end).get_unchecked_mut(slice) }
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

unsafe impl<T> SoaSliceIndex<SoaSlice<T>> for (ops::Bound<usize>, ops::Bound<usize>)
where
    T: Soa,
{
    type Ref<'a>
        = T::Slices<'a>
    where
        SoaSlice<T>: 'a;

    type RefMut<'a>
        = T::SlicesMut<'a>
    where
        SoaSlice<T>: 'a;

    #[inline]
    fn get(self, slice: &SoaSlice<T>) -> Option<Self::Ref<'_>> {
        into_range(slice.len(), self)?.get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut SoaSlice<T>) -> Option<Self::RefMut<'_>> {
        into_range(slice.len(), self)?.get_mut(slice)
    }

    #[inline]
    fn index(self, slice: &SoaSlice<T>) -> Self::Ref<'_> {
        into_slice_range(slice.len(), self).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut SoaSlice<T>) -> Self::RefMut<'_> {
        into_slice_range(slice.len(), self).index_mut(slice)
    }

    type Ptr = T::SlicePtrs;

    type MutPtr = T::SliceMutPtrs;

    #[inline]
    unsafe fn get_unchecked(self, slice: *const SoaSlice<T>) -> Self::Ptr {
        unsafe { into_range_unchecked(slice.len(), self).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T>) -> Self::MutPtr {
        unsafe { into_range_unchecked(slice.len(), self).get_unchecked_mut(slice) }
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
