use core::ops;

use crate::{
    slice::assert::{
        slice_end_index_len_fail, slice_end_index_overflow_fail, slice_index_order_fail,
        slice_index_usize_fail, slice_start_index_len_fail, slice_start_index_overflow_fail,
    },
    traits::{MutPtrs, Ptrs, SliceMutPtrs, SlicePtrs, Soa, SoaContext},
};

pub unsafe trait SoaSliceIndex<T>: private_slice_index::Sealed
where
    T: Soa + ?Sized,
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
        slices: SlicePtrs<'context, T>,
    ) -> Self::Ptrs<'context>;

    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: SliceMutPtrs<'context, T>,
    ) -> Self::MutPtrs<'context>;
}

unsafe impl<T> SoaSliceIndex<T> for usize
where
    T: Soa + ?Sized,
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
        let len = context.slice_ptrs_len(&slices);
        if self >= len {
            return None;
        }

        let ptrs = unsafe { SoaSliceIndex::<T>::get_unchecked(self, context, slices) };
        let refs = unsafe { T::ptrs_to_refs(context, ptrs) };
        Some(refs)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        let len = context.slice_mut_ptrs_len(&slices);
        if self >= len {
            return None;
        }

        let ptrs = unsafe { SoaSliceIndex::<T>::get_unchecked_mut(self, context, slices) };
        let refs = unsafe { T::ptrs_to_refs_mut(context, ptrs) };
        Some(refs)
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

    type Ptrs<'context> = Ptrs<'context, T>;

    type MutPtrs<'context> = MutPtrs<'context, T>;

    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: SlicePtrs<'context, T>,
    ) -> Self::Ptrs<'context> {
        let len = context.slice_ptrs_len(&slices);
        debug_assert!(
            self < len,
            "slice::get_unchecked requires that the index is within the slice",
        );

        let ptrs = context.slice_ptrs_as_ptrs(slices);
        unsafe { context.ptrs_add(ptrs, self) }
    }

    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: SliceMutPtrs<'context, T>,
    ) -> Self::MutPtrs<'context> {
        let len = context.slice_mut_ptrs_len(&slices);
        debug_assert!(
            self < len,
            "slice::get_unchecked_mut requires that the index is within the slice",
        );

        let ptrs = context.slice_mut_ptrs_as_ptrs(slices);
        unsafe { context.ptrs_add_mut(ptrs, self) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::Range<usize>
where
    T: Soa + ?Sized,
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
        let Self { start, end } = self;
        let slices = T::slices_as_slice_ptrs(context, slices);
        let len = context.slice_ptrs_len(&slices);
        if start > end || end > len {
            return None;
        }

        let slices = unsafe { SoaSliceIndex::<T>::get_unchecked(self, context, slices) };
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        Some(slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let Self { start, end } = self;
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        let len = context.slice_mut_ptrs_len(&slices);
        if start > end || end > len {
            return None;
        }

        let slices = unsafe { SoaSliceIndex::<T>::get_unchecked_mut(self, context, slices) };
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        Some(slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let Self { start, end } = self;
        let len = T::slices_len(context, &slices);
        if start > end {
            slice_index_order_fail(start, end);
        } else if end > len {
            slice_end_index_len_fail(end, len);
        }

        let slices = T::slices_as_slice_ptrs(context, slices);
        let slices = unsafe { SoaSliceIndex::<T>::get_unchecked(self, context, slices) };
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        let Self { start, end } = self;
        let len = T::slices_mut_len(context, &slices);
        if start > end {
            slice_index_order_fail(start, end);
        } else if end > len {
            slice_end_index_len_fail(end, len);
        }

        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        let slices = unsafe { SoaSliceIndex::<T>::get_unchecked_mut(self, context, slices) };
        unsafe { T::slice_mut_ptrs_to_slices(context, slices) }
    }

    type Ptrs<'context> = SlicePtrs<'context, T>;

    type MutPtrs<'context> = SliceMutPtrs<'context, T>;

    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: SlicePtrs<'context, T>,
    ) -> Self::Ptrs<'context> {
        let Self { start, end } = self;
        let len = context.slice_ptrs_len(&slices);
        debug_assert!(
            end >= start && end <= len,
            "slice::get_unchecked requires that the range is within the slice",
        );

        let ptrs = context.slice_ptrs_as_ptrs(slices);
        let ptrs = unsafe { context.ptrs_add(ptrs, start) };
        let new_len = unsafe { end.unchecked_sub(start) };
        context.slice_ptrs_from_raw_parts(ptrs, new_len)
    }

    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: SliceMutPtrs<'context, T>,
    ) -> Self::MutPtrs<'context> {
        let Self { start, end } = self;
        let len = context.slice_mut_ptrs_len(&slices);
        debug_assert!(
            end >= start && end <= len,
            "slice::get_unchecked_mut requires that the range is within the slice",
        );

        let ptrs = context.slice_mut_ptrs_as_ptrs(slices);
        let ptrs = unsafe { context.ptrs_add_mut(ptrs, start) };
        let new_len = unsafe { end.unchecked_sub(start) };
        context.slice_mut_ptrs_from_raw_parts(ptrs, new_len)
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeTo<usize>
where
    T: Soa + ?Sized,
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
        let Self { end } = self;
        SoaSliceIndex::<T>::get(0..end, context, slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let Self { end } = self;
        SoaSliceIndex::<T>::get_mut(0..end, context, slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let Self { end } = self;
        SoaSliceIndex::<T>::index(0..end, context, slices)
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        let Self { end } = self;
        SoaSliceIndex::<T>::index_mut(0..end, context, slices)
    }

    type Ptrs<'context> = SlicePtrs<'context, T>;

    type MutPtrs<'context> = SliceMutPtrs<'context, T>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: SlicePtrs<'context, T>,
    ) -> Self::Ptrs<'context> {
        let Self { end } = self;
        unsafe { SoaSliceIndex::<T>::get_unchecked(0..end, context, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: SliceMutPtrs<'context, T>,
    ) -> Self::MutPtrs<'context> {
        let Self { end } = self;
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(0..end, context, slices) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeFrom<usize>
where
    T: Soa + ?Sized,
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
        let Self { start } = self;
        let len = T::slices_len(context, &slices);
        SoaSliceIndex::<T>::get(start..len, context, slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let Self { start } = self;
        let len = T::slices_mut_len(context, &slices);
        SoaSliceIndex::<T>::get_mut(start..len, context, slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let Self { start } = self;
        let len = T::slices_len(context, &slices);
        if start > len {
            slice_start_index_len_fail(start, len);
        }

        let slices = T::slices_as_slice_ptrs(context, slices);
        let slices = unsafe { SoaSliceIndex::<T>::get_unchecked(self, context, slices) };
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        let Self { start } = self;
        let len = T::slices_mut_len(context, &slices);
        if start > len {
            slice_start_index_len_fail(start, len);
        }

        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        let slices = unsafe { SoaSliceIndex::<T>::get_unchecked_mut(self, context, slices) };
        unsafe { T::slice_mut_ptrs_to_slices(context, slices) }
    }

    type Ptrs<'context> = SlicePtrs<'context, T>;

    type MutPtrs<'context> = SliceMutPtrs<'context, T>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: SlicePtrs<'context, T>,
    ) -> Self::Ptrs<'context> {
        let Self { start } = self;
        let len = context.slice_ptrs_len(&slices);
        unsafe { SoaSliceIndex::<T>::get_unchecked(start..len, context, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: SliceMutPtrs<'context, T>,
    ) -> Self::MutPtrs<'context> {
        let Self { start } = self;
        let len = context.slice_mut_ptrs_len(&slices);
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(start..len, context, slices) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeFull
where
    T: Soa + ?Sized,
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

    type Ptrs<'context> = SlicePtrs<'context, T>;

    type MutPtrs<'context> = SliceMutPtrs<'context, T>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        _context: &'context T::Context,
        slices: SlicePtrs<'context, T>,
    ) -> Self::Ptrs<'context> {
        slices
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        _context: &'context T::Context,
        slices: SliceMutPtrs<'context, T>,
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
    T: Soa + ?Sized,
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
        let range = range_into_slice_range(self);
        SoaSliceIndex::<T>::get(range, context, slices)
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
        let range = range_into_slice_range(self);
        SoaSliceIndex::<T>::get_mut(range, context, slices)
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
        let range = range_into_slice_range(self);
        SoaSliceIndex::<T>::index(range, context, slices)
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
        let range = range_into_slice_range(self);
        SoaSliceIndex::<T>::index_mut(range, context, slices)
    }

    type Ptrs<'context> = SlicePtrs<'context, T>;

    type MutPtrs<'context> = SliceMutPtrs<'context, T>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: SlicePtrs<'context, T>,
    ) -> Self::Ptrs<'context> {
        let range = range_into_slice_range(self);
        unsafe { SoaSliceIndex::<T>::get_unchecked(range, context, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: SliceMutPtrs<'context, T>,
    ) -> Self::MutPtrs<'context> {
        let range = range_into_slice_range(self);
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(range, context, slices) }
    }
}

unsafe impl<T> SoaSliceIndex<T> for ops::RangeToInclusive<usize>
where
    T: Soa + ?Sized,
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
        let Self { end } = self;
        SoaSliceIndex::<T>::get(0..=end, context, slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let Self { end } = self;
        SoaSliceIndex::<T>::get_mut(0..=end, context, slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let Self { end } = self;
        SoaSliceIndex::<T>::index(0..=end, context, slices)
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        let Self { end } = self;
        SoaSliceIndex::<T>::index_mut(0..=end, context, slices)
    }

    type Ptrs<'context> = SlicePtrs<'context, T>;

    type MutPtrs<'context> = SliceMutPtrs<'context, T>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: SlicePtrs<'context, T>,
    ) -> Self::Ptrs<'context> {
        let Self { end } = self;
        unsafe { SoaSliceIndex::<T>::get_unchecked(0..=end, context, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: SliceMutPtrs<'context, T>,
    ) -> Self::MutPtrs<'context> {
        let Self { end } = self;
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(0..=end, context, slices) }
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
    T: Soa + ?Sized,
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
        let range = into_range(len, self)?;
        SoaSliceIndex::<T>::get(range, context, slices)
    }

    #[inline]
    fn get_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Option<Self::RefsMut<'context, 'a>> {
        let len = T::slices_mut_len(context, &slices);
        let range = into_range(len, self)?;
        SoaSliceIndex::<T>::get_mut(range, context, slices)
    }

    #[inline]
    fn index<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::Slices<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let len = T::slices_len(context, &slices);
        let range = into_slice_range(len, self);
        SoaSliceIndex::<T>::index(range, context, slices)
    }

    #[inline]
    fn index_mut<'context, 'a>(
        self,
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self::RefsMut<'context, 'a> {
        let len = T::slices_mut_len(context, &slices);
        let range = into_slice_range(len, self);
        SoaSliceIndex::<T>::index_mut(range, context, slices)
    }

    type Ptrs<'context> = SlicePtrs<'context, T>;

    type MutPtrs<'context> = SliceMutPtrs<'context, T>;

    #[inline]
    unsafe fn get_unchecked<'context>(
        self,
        context: &'context T::Context,
        slices: SlicePtrs<'context, T>,
    ) -> Self::Ptrs<'context> {
        let len = context.slice_ptrs_len(&slices);
        let range = into_range_unchecked(len, self);
        unsafe { SoaSliceIndex::<T>::get_unchecked(range, context, slices) }
    }

    #[inline]
    unsafe fn get_unchecked_mut<'context>(
        self,
        context: &'context T::Context,
        slices: SliceMutPtrs<'context, T>,
    ) -> Self::MutPtrs<'context> {
        let len = context.slice_mut_ptrs_len(&slices);
        let range = into_range_unchecked(len, self);
        unsafe { SoaSliceIndex::<T>::get_unchecked_mut(range, context, slices) }
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

pub trait IndexHelper<'c, 'a, T>
where
    Self: SoaSliceIndex<T, Refs<'c, 'a> = &'a Self::Output>,
    T: Soa + ?Sized + 'a,
{
    type Output: ?Sized + 'a;
}

impl<'c, 'a, T, I, U> IndexHelper<'c, 'a, T> for I
where
    U: ?Sized + 'a,
    T: Soa + ?Sized + 'a,
    I: SoaSliceIndex<T, Refs<'c, 'a> = &'a U>,
{
    type Output = U;
}

pub trait IndexHelperMut<'c, 'a, T>
where
    Self: IndexHelper<'c, 'a, T> + SoaSliceIndex<T, RefsMut<'c, 'a> = &'a mut Self::Output>,
    T: Soa + ?Sized + 'a,
{
}

impl<'c, 'a, T, I, U> IndexHelperMut<'c, 'a, T> for I
where
    U: ?Sized + 'a,
    T: Soa + ?Sized + 'a,
    I: IndexHelper<'c, 'a, T, Output = U> + SoaSliceIndex<T, RefsMut<'c, 'a> = &'a mut U>,
{
}

/// Just a copy of unstable [`core::slice::range`]
#[must_use]
#[track_caller]
#[doc(hidden)]
pub fn range<R>(range: R, bounds: ops::RangeTo<usize>) -> ops::Range<usize>
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
