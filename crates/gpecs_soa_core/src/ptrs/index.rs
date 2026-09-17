use core::ops;

use crate::{
    ptrs::{get, get_mut, get_unchecked, get_unchecked_mut, index, index_mut},
    traits::{MutPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs},
};

pub unsafe trait SlicePtrsIndex<T>: private::Sealed
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'a>;

    unsafe fn get_unchecked<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Self::Ptrs<'a>;

    fn get_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Option<Self::Ptrs<'a>>;

    fn index_ptrs<'a>(self, context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Self::Ptrs<'a>;

    type MutPtrs<'a>;

    unsafe fn get_unchecked_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a>;

    fn get_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Option<Self::MutPtrs<'a>>;

    fn index_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a>;
}

unsafe impl<T> SlicePtrsIndex<T> for usize
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'a> = Ptrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Self::Ptrs<'a> {
        unsafe { get_offset_unchecked::<T>(context, slices, self) }
    }

    #[inline]
    fn get_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Option<Self::Ptrs<'a>> {
        let len = context.slice_ptrs_len(&slices);
        if self >= len {
            return None;
        }

        let ptrs = unsafe { get_unchecked::<T, _>(context, slices, self) };
        Some(ptrs)
    }

    #[inline]
    fn index_ptrs<'a>(self, context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Self::Ptrs<'a> {
        let len = context.slice_ptrs_len(&slices);
        if self >= len {
            slice_index_usize_fail(len, self)
        }

        unsafe { get_unchecked::<T, _>(context, slices, self) }
    }

    type MutPtrs<'a> = MutPtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        unsafe { get_offset_unchecked_mut::<T>(context, slices, self) }
    }

    #[inline]
    fn get_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Option<Self::MutPtrs<'a>> {
        let len = context.mut_slice_ptrs_len(&slices);
        if self >= len {
            return None;
        }

        let ptrs = unsafe { get_unchecked_mut::<T, _>(context, slices, self) };
        Some(ptrs)
    }

    #[inline]
    fn index_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let len = context.mut_slice_ptrs_len(&slices);
        if self >= len {
            slice_index_usize_fail(len, self)
        }

        unsafe { get_unchecked_mut::<T, _>(context, slices, self) }
    }
}

unsafe impl<T> SlicePtrsIndex<T> for ops::Range<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'a> = SlicePtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Self::Ptrs<'a> {
        let Self { start, end } = self;
        let new_len = unsafe { end.unchecked_sub(start) };
        unsafe { get_offset_len_unchecked::<T>(context, slices, start, new_len) }
    }

    #[inline]
    fn get_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Option<Self::Ptrs<'a>> {
        let Self { start, end } = self;
        let slices_len = context.slice_ptrs_len(&slices);

        let new_len = end.checked_sub(start)?;
        if end > slices_len {
            return None;
        }

        let slices = unsafe { get_offset_len_unchecked::<T>(context, slices, start, new_len) };
        Some(slices)
    }

    #[inline]
    fn index_ptrs<'a>(self, context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Self::Ptrs<'a> {
        let Self { start, end } = self;
        let slices_len = context.slice_ptrs_len(&slices);

        let Some(new_len) = end.checked_sub(start) else {
            slice_index_fail(start, end, slices_len)
        };
        if end > slices_len {
            slice_index_fail(start, end, slices_len)
        }

        unsafe { get_offset_len_unchecked::<T>(context, slices, start, new_len) }
    }

    type MutPtrs<'a> = SliceMutPtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let Self { start, end } = self;
        let new_len = unsafe { end.unchecked_sub(start) };
        unsafe { get_offset_len_unchecked_mut::<T>(context, slices, start, new_len) }
    }

    #[inline]
    fn get_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Option<Self::MutPtrs<'a>> {
        let Self { start, end } = self;
        let slices_len = context.mut_slice_ptrs_len(&slices);

        let new_len = end.checked_sub(start)?;
        if end > slices_len {
            return None;
        }

        let slices = unsafe { get_offset_len_unchecked_mut::<T>(context, slices, start, new_len) };
        Some(slices)
    }

    #[inline]
    fn index_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let Self { start, end } = self;
        let slices_len = context.mut_slice_ptrs_len(&slices);

        let Some(new_len) = end.checked_sub(start) else {
            slice_index_fail(start, end, slices_len)
        };
        if end > slices_len {
            slice_index_fail(start, end, slices_len)
        }

        unsafe { get_offset_len_unchecked_mut::<T>(context, slices, start, new_len) }
    }
}

unsafe impl<T> SlicePtrsIndex<T> for ops::RangeTo<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'a> = SlicePtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Self::Ptrs<'a> {
        let Self { end } = self;
        unsafe { get_unchecked::<T, _>(context, slices, 0..end) }
    }

    #[inline]
    fn get_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Option<Self::Ptrs<'a>> {
        let Self { end } = self;
        get::<T, _>(context, slices, 0..end)
    }

    #[inline]
    fn index_ptrs<'a>(self, context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Self::Ptrs<'a> {
        let Self { end } = self;
        index::<T, _>(context, slices, 0..end)
    }

    type MutPtrs<'a> = SliceMutPtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let Self { end } = self;
        unsafe { get_unchecked_mut::<T, _>(context, slices, 0..end) }
    }

    #[inline]
    fn get_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Option<Self::MutPtrs<'a>> {
        let Self { end } = self;
        get_mut::<T, _>(context, slices, 0..end)
    }

    #[inline]
    fn index_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let Self { end } = self;
        index_mut::<T, _>(context, slices, 0..end)
    }
}

unsafe impl<T> SlicePtrsIndex<T> for ops::RangeFrom<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'a> = SlicePtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Self::Ptrs<'a> {
        let Self { start } = self;
        let len = context.slice_ptrs_len(&slices);
        unsafe { get_unchecked::<T, _>(context, slices, start..len) }
    }

    #[inline]
    fn get_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Option<Self::Ptrs<'a>> {
        let Self { start } = self;
        let len = context.slice_ptrs_len(&slices);
        get::<T, _>(context, slices, start..len)
    }

    #[inline]
    fn index_ptrs<'a>(self, context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Self::Ptrs<'a> {
        let Self { start } = self;
        let len = context.slice_ptrs_len(&slices);
        if start > len {
            slice_index_fail(start, len, len)
        }

        let new_len = unsafe { len.unchecked_sub(start) };
        unsafe { get_offset_len_unchecked::<T>(context, slices, start, new_len) }
    }

    type MutPtrs<'a> = SliceMutPtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let Self { start } = self;
        let len = context.mut_slice_ptrs_len(&slices);
        unsafe { get_unchecked_mut::<T, _>(context, slices, start..len) }
    }

    #[inline]
    fn get_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Option<Self::MutPtrs<'a>> {
        let Self { start } = self;
        let len = context.mut_slice_ptrs_len(&slices);
        get_mut::<T, _>(context, slices, start..len)
    }

    #[inline]
    fn index_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let Self { start } = self;
        let len = context.mut_slice_ptrs_len(&slices);
        if start > len {
            slice_index_fail(start, len, len)
        }

        let new_len = unsafe { len.unchecked_sub(start) };
        unsafe { get_offset_len_unchecked_mut::<T>(context, slices, start, new_len) }
    }
}

unsafe impl<T> SlicePtrsIndex<T> for ops::RangeFull
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'a> = SlicePtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked<'a>(
        self,
        _context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Self::Ptrs<'a> {
        slices
    }

    #[inline]
    fn get_ptrs<'a>(
        self,
        _context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Option<Self::Ptrs<'a>> {
        Some(slices)
    }

    #[inline]
    fn index_ptrs<'a>(self, _context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Self::Ptrs<'a> {
        slices
    }

    type MutPtrs<'a> = SliceMutPtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'a>(
        self,
        _context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        slices
    }

    #[inline]
    fn get_mut_ptrs<'a>(
        self,
        _context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Option<Self::MutPtrs<'a>> {
        Some(slices)
    }

    #[inline]
    fn index_mut_ptrs<'a>(
        self,
        _context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
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

unsafe impl<T> SlicePtrsIndex<T> for ops::RangeInclusive<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'a> = SlicePtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Self::Ptrs<'a> {
        let range = range_into_slice_range(self);
        unsafe { get_unchecked::<T, _>(context, slices, range) }
    }

    #[inline]
    fn get_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Option<Self::Ptrs<'a>> {
        let end = *self.end();
        let len = context.slice_ptrs_len(&slices);
        if end >= len {
            return None;
        }

        let range = range_into_slice_range(self);
        get::<T, _>(context, slices, range)
    }

    #[inline]
    fn index_ptrs<'a>(self, context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Self::Ptrs<'a> {
        let start = *self.start();
        let end = *self.end();
        let len = context.slice_ptrs_len(&slices);
        if end >= len {
            slice_index_fail(start, end, len)
        }

        let range = range_into_slice_range(self);
        index::<T, _>(context, slices, range)
    }

    type MutPtrs<'a> = SliceMutPtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let range = range_into_slice_range(self);
        unsafe { get_unchecked_mut::<T, _>(context, slices, range) }
    }

    #[inline]
    fn get_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Option<Self::MutPtrs<'a>> {
        let end = *self.end();
        let len = context.mut_slice_ptrs_len(&slices);
        if end >= len {
            return None;
        }

        let range = range_into_slice_range(self);
        get_mut::<T, _>(context, slices, range)
    }

    #[inline]
    fn index_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let start = *self.start();
        let end = *self.end();
        let len = context.mut_slice_ptrs_len(&slices);
        if end >= len {
            slice_index_fail(start, end, len)
        }

        let range = range_into_slice_range(self);
        index_mut::<T, _>(context, slices, range)
    }
}

unsafe impl<T> SlicePtrsIndex<T> for ops::RangeToInclusive<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'a> = SlicePtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Self::Ptrs<'a> {
        let Self { end } = self;
        unsafe { get_unchecked::<T, _>(context, slices, 0..=end) }
    }

    #[inline]
    fn get_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Option<Self::Ptrs<'a>> {
        let Self { end } = self;
        get::<T, _>(context, slices, 0..=end)
    }

    #[inline]
    fn index_ptrs<'a>(self, context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Self::Ptrs<'a> {
        let Self { end } = self;
        index::<T, _>(context, slices, 0..=end)
    }

    type MutPtrs<'a> = SliceMutPtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let Self { end } = self;
        unsafe { get_unchecked_mut::<T, _>(context, slices, 0..=end) }
    }

    #[inline]
    fn get_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Option<Self::MutPtrs<'a>> {
        let Self { end } = self;
        get_mut::<T, _>(context, slices, 0..=end)
    }

    #[inline]
    fn index_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let Self { end } = self;
        index_mut::<T, _>(context, slices, 0..=end)
    }
}

/// Copy of private [`core::slice::index::into_range_unchecked()`].
const fn into_range_unchecked(
    len: usize,
    (start, end): (ops::Bound<usize>, ops::Bound<usize>),
) -> ops::Range<usize> {
    let start = match start {
        ops::Bound::Included(i) => i,
        ops::Bound::Excluded(i) => i + 1,
        ops::Bound::Unbounded => 0,
    };
    let end = match end {
        ops::Bound::Included(i) => i + 1,
        ops::Bound::Excluded(i) => i,
        ops::Bound::Unbounded => len,
    };
    start..end
}

/// Copy of private [`core::slice::index::try_into_slice_range()`].
#[inline]
const fn try_into_slice_range(
    len: usize,
    (start, end): (ops::Bound<usize>, ops::Bound<usize>),
) -> Option<ops::Range<usize>> {
    let end = match end {
        ops::Bound::Included(end) if end >= len => return None,
        // Cannot overflow because `end < len` implies `end < usize::MAX`.
        ops::Bound::Included(end) => end + 1,

        ops::Bound::Excluded(end) if end > len => return None,
        ops::Bound::Excluded(end) => end,

        ops::Bound::Unbounded => len,
    };

    let start = match start {
        ops::Bound::Excluded(start) if start >= end => return None,
        // Cannot overflow because `start < end` implies `start < usize::MAX`.
        ops::Bound::Excluded(start) => start + 1,

        ops::Bound::Included(start) if start > end => return None,
        ops::Bound::Included(start) => start,

        ops::Bound::Unbounded => 0,
    };

    Some(start..end)
}

/// Copy of private [`core::slice::index::into_slice_range()`].
#[inline]
fn into_slice_range(
    len: usize,
    (start, end): (ops::Bound<usize>, ops::Bound<usize>),
) -> ops::Range<usize> {
    let end = match end {
        ops::Bound::Included(end) if end >= len => slice_index_fail(0, end, len),
        // Cannot overflow because `end < len` implies `end < usize::MAX`.
        ops::Bound::Included(end) => end + 1,

        ops::Bound::Excluded(end) if end > len => slice_index_fail(0, end, len),
        ops::Bound::Excluded(end) => end,

        ops::Bound::Unbounded => len,
    };

    let start = match start {
        ops::Bound::Excluded(start) if start >= end => slice_index_fail(start, end, len),
        // Cannot overflow because `start < end` implies `start < usize::MAX`.
        ops::Bound::Excluded(start) => start + 1,

        ops::Bound::Included(start) if start > end => slice_index_fail(start, end, len),
        ops::Bound::Included(start) => start,

        ops::Bound::Unbounded => 0,
    };

    start..end
}

unsafe impl<T> SlicePtrsIndex<T> for (ops::Bound<usize>, ops::Bound<usize>)
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'a> = SlicePtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Self::Ptrs<'a> {
        let len = context.slice_ptrs_len(&slices);
        let range = into_range_unchecked(len, self);
        unsafe { get_unchecked::<T, _>(context, slices, range) }
    }

    #[inline]
    fn get_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SlicePtrs<'a, T>,
    ) -> Option<Self::Ptrs<'a>> {
        let len = context.slice_ptrs_len(&slices);
        let range = try_into_slice_range(len, self)?;
        get::<T, _>(context, slices, range)
    }

    #[inline]
    fn index_ptrs<'a>(self, context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Self::Ptrs<'a> {
        let len = context.slice_ptrs_len(&slices);
        let range = into_slice_range(len, self);
        index::<T, _>(context, slices, range)
    }

    type MutPtrs<'a> = SliceMutPtrs<'a, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let len = context.mut_slice_ptrs_len(&slices);
        let range = into_range_unchecked(len, self);
        unsafe { get_unchecked_mut::<T, _>(context, slices, range) }
    }

    #[inline]
    fn get_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Option<Self::MutPtrs<'a>> {
        let len = context.mut_slice_ptrs_len(&slices);
        let range = try_into_slice_range(len, self)?;
        get_mut::<T, _>(context, slices, range)
    }

    #[inline]
    fn index_mut_ptrs<'a>(
        self,
        context: &'a T::Context,
        slices: SliceMutPtrs<'a, T>,
    ) -> Self::MutPtrs<'a> {
        let len = context.mut_slice_ptrs_len(&slices);
        let range = into_slice_range(len, self);
        index_mut::<T, _>(context, slices, range)
    }
}

mod private {
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

#[inline]
unsafe fn get_offset_unchecked<'a, T>(
    context: &'a T::Context,
    slices: SlicePtrs<'a, T>,
    offset: usize,
) -> Ptrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    let ptrs = context.slice_ptrs_as_ptrs(slices);
    unsafe { context.ptrs_add(ptrs, offset) }
}

#[inline]
unsafe fn get_offset_unchecked_mut<'a, T>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
    offset: usize,
) -> MutPtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    let ptrs = context.mut_slice_ptrs_as_mut_ptrs(slices);
    unsafe { context.mut_ptrs_add(ptrs, offset) }
}

/// Copy of private `core::slice::index::get_offset_len_noubcheck()`.
#[inline]
unsafe fn get_offset_len_unchecked<'a, T>(
    context: &'a T::Context,
    slices: SlicePtrs<'a, T>,
    offset: usize,
    len: usize,
) -> SlicePtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    let data = unsafe { get_offset_unchecked::<T>(context, slices, offset) };
    context.slice_ptrs_from_raw_parts(data, len)
}

/// Copy of private `core::slice::index::get_offset_len_mut_noubcheck()`.
#[inline]
unsafe fn get_offset_len_unchecked_mut<'a, T>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
    offset: usize,
    len: usize,
) -> SliceMutPtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    let data = unsafe { get_offset_unchecked_mut::<T>(context, slices, offset) };
    context.mut_slice_ptrs_from_raw_parts(data, len)
}

/// Copy of [`core::slice::try_range()`].
#[must_use]
#[doc(hidden)]
pub fn try_range<R>(range: R, bounds: ops::RangeTo<usize>) -> Option<ops::Range<usize>>
where
    R: ops::RangeBounds<usize>,
{
    let len = bounds.end;
    let start = ops_bound_copied(range.start_bound());
    let end = ops_bound_copied(range.end_bound());
    try_into_slice_range(len, (start, end))
}

/// Copy of [`core::slice::range()`].
#[must_use]
#[track_caller]
#[doc(hidden)]
pub fn range<R>(range: R, bounds: ops::RangeTo<usize>) -> ops::Range<usize>
where
    R: ops::RangeBounds<usize>,
{
    let len = bounds.end;
    let start = ops_bound_copied(range.start_bound());
    let end = ops_bound_copied(range.end_bound());
    into_slice_range(len, (start, end))
}

/// Copy of [`core::ops::Bound::copied()`].
#[must_use]
const fn ops_bound_copied<T>(bound: ops::Bound<&T>) -> ops::Bound<T>
where
    T: Copy,
{
    match bound {
        ops::Bound::Unbounded => ops::Bound::Unbounded,
        ops::Bound::Included(&x) => ops::Bound::Included(x),
        ops::Bound::Excluded(&x) => ops::Bound::Excluded(x),
    }
}

#[cfg_attr(not(panic = "immediate-abort"), inline(never), cold)]
#[cfg_attr(panic = "immediate-abort", inline)]
#[track_caller]
fn slice_index_usize_fail(len: usize, index: usize) -> ! {
    panic!("index out of bounds: the len of SoA slice is {len} but the index is {index}")
}

/// Copy of private `core::slice::index::slice_index_fail()`.
#[cfg_attr(not(panic = "immediate-abort"), inline(never), cold)]
#[cfg_attr(panic = "immediate-abort", inline)]
#[track_caller]
fn slice_index_fail(start: usize, end: usize, len: usize) -> ! {
    assert!(
        start <= len,
        "range start index {start} out of range for SoA slice of length {len}",
    );
    assert!(
        end <= len,
        "range end index {end} out of range for SoA slice of length {len}",
    );
    assert!(
        start <= end,
        "SoA slice index starts at {start} but ends at {end}",
    );

    // Only reachable if the range was a `RangeInclusive` or a
    // `RangeToInclusive`, with `end == len`.
    panic!("range end index {end} out of range for SoA slice of length {len}")
}
