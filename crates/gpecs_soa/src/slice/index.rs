use core::ops;

use crate::traits::{
    MutPtrs, Ptrs, RawSoa, RawSoaContext, Refs, RefsMut, SliceMutPtrs, SlicePtrs, Slices,
    SlicesMut, Soa, SoaContext,
};

pub unsafe trait SoaSlicePtrsIndex<T>: private_slice_index::Sealed
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'ctx>;

    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx>;

    fn get_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>>;

    fn index_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx>;

    type MutPtrs<'ctx>;

    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx>;

    fn get_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>>;

    fn index_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx>;
}

pub unsafe trait SoaSlicesIndex<'a, T>: SoaSlicePtrsIndex<T>
where
    T: Soa<'a> + ?Sized,
{
    type Refs<'ctx>;

    fn get<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>>;

    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx>;

    type RefsMut<'ctx>;

    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>>;

    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx>;
}

unsafe impl<T> SoaSlicePtrsIndex<T> for usize
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'ctx> = Ptrs<'ctx, T>;

    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let len = context.slice_ptrs_len(&slices);
        debug_assert!(
            self < len,
            "slice::get_unchecked requires that the index is within the slice",
        );

        let ptrs = context.slice_ptrs_as_ptrs(slices);
        unsafe { context.ptrs_add(ptrs, self) }
    }

    #[inline]
    fn get_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>> {
        let len = context.slice_ptrs_len(&slices);
        if self >= len {
            return None;
        }

        let ptrs = unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(self, context, slices) };
        Some(ptrs)
    }

    #[inline]
    fn index_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let len = context.slice_ptrs_len(&slices);
        match SoaSlicePtrsIndex::<T>::get_ptrs(self, context, slices) {
            Some(ptrs) => ptrs,
            None => slice_index_usize_fail(len, self),
        }
    }

    type MutPtrs<'ctx> = MutPtrs<'ctx, T>;

    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let len = context.mut_slice_ptrs_len(&slices);
        debug_assert!(
            self < len,
            "slice::get_unchecked_mut requires that the index is within the slice",
        );

        let ptrs = context.mut_slice_ptrs_as_ptrs(slices);
        unsafe { context.ptrs_add_mut(ptrs, self) }
    }

    #[inline]
    fn get_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>> {
        let len = context.mut_slice_ptrs_len(&slices);
        if self >= len {
            return None;
        }

        let ptrs = unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(self, context, slices) };
        Some(ptrs)
    }

    #[inline]
    fn index_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let len = context.mut_slice_ptrs_len(&slices);
        match SoaSlicePtrsIndex::<T>::get_mut_ptrs(self, context, slices) {
            Some(ptrs) => ptrs,
            None => slice_index_usize_fail(len, self),
        }
    }
}

unsafe impl<'a, T> SoaSlicesIndex<'a, T> for usize
where
    T: Soa<'a> + ?Sized,
{
    type Refs<'ctx> = Refs<'ctx, 'a, T>;

    #[inline]
    fn get<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let ptrs = SoaSlicePtrsIndex::<T>::get_ptrs(self, context, slices)?;
        let refs = unsafe { context.ptrs_to_refs(ptrs) };
        Some(refs)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let ptrs = SoaSlicePtrsIndex::<T>::index_ptrs(self, context, slices);
        unsafe { context.ptrs_to_refs(ptrs) }
    }

    type RefsMut<'ctx> = RefsMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let ptrs = SoaSlicePtrsIndex::<T>::get_mut_ptrs(self, context, slices)?;
        let refs = unsafe { context.mut_ptrs_to_mut_refs(ptrs) };
        Some(refs)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let ptrs = SoaSlicePtrsIndex::<T>::index_mut_ptrs(self, context, slices);
        unsafe { context.mut_ptrs_to_mut_refs(ptrs) }
    }
}

unsafe impl<T> SoaSlicePtrsIndex<T> for ops::Range<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'ctx> = SlicePtrs<'ctx, T>;

    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
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

    #[inline]
    fn get_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>> {
        let Self { start, end } = self;
        let len = context.slice_ptrs_len(&slices);
        if start > end || end > len {
            return None;
        }

        let slices = unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(self, context, slices) };
        Some(slices)
    }

    #[inline]
    fn index_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let Self { start, end } = self;
        let len = context.slice_ptrs_len(&slices);
        if start > end {
            slice_index_order_fail(start, end);
        } else if end > len {
            slice_end_index_len_fail(end, len);
        }

        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(self, context, slices) }
    }

    type MutPtrs<'ctx> = SliceMutPtrs<'ctx, T>;

    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let Self { start, end } = self;
        let len = context.mut_slice_ptrs_len(&slices);
        debug_assert!(
            end >= start && end <= len,
            "slice::get_unchecked_mut requires that the range is within the slice",
        );

        let ptrs = context.mut_slice_ptrs_as_ptrs(slices);
        let ptrs = unsafe { context.ptrs_add_mut(ptrs, start) };
        let new_len = unsafe { end.unchecked_sub(start) };
        context.mut_slice_ptrs_from_raw_parts(ptrs, new_len)
    }

    #[inline]
    fn get_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>> {
        let Self { start, end } = self;
        let len = context.mut_slice_ptrs_len(&slices);
        if start > end || end > len {
            return None;
        }

        let slices = unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(self, context, slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let Self { start, end } = self;
        let len = context.mut_slice_ptrs_len(&slices);
        if start > end {
            slice_index_order_fail(start, end);
        } else if end > len {
            slice_end_index_len_fail(end, len);
        }

        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(self, context, slices) }
    }
}

unsafe impl<'a, T> SoaSlicesIndex<'a, T> for ops::Range<usize>
where
    T: Soa<'a> + ?Sized,
{
    type Refs<'ctx> = Slices<'ctx, 'a, T>;

    #[inline]
    fn get<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_ptrs(self, context, slices)?;
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_ptrs(self, context, slices);
        unsafe { context.slice_ptrs_to_slices(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_mut_ptrs(self, context, slices)?;
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_mut_ptrs(self, context, slices);
        unsafe { context.mut_slice_ptrs_to_mut_slices(slices) }
    }
}

unsafe impl<T> SoaSlicePtrsIndex<T> for ops::RangeTo<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'ctx> = SlicePtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let Self { end } = self;
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(0..end, context, slices) }
    }

    #[inline]
    fn get_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>> {
        let Self { end } = self;
        SoaSlicePtrsIndex::<T>::get_ptrs(0..end, context, slices)
    }

    #[inline]
    fn index_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let Self { end } = self;
        SoaSlicePtrsIndex::<T>::index_ptrs(0..end, context, slices)
    }

    type MutPtrs<'ctx> = SliceMutPtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let Self { end } = self;
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(0..end, context, slices) }
    }

    #[inline]
    fn get_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>> {
        let Self { end } = self;
        SoaSlicePtrsIndex::<T>::get_mut_ptrs(0..end, context, slices)
    }

    #[inline]
    fn index_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let Self { end } = self;
        SoaSlicePtrsIndex::<T>::index_mut_ptrs(0..end, context, slices)
    }
}

unsafe impl<'a, T> SoaSlicesIndex<'a, T> for ops::RangeTo<usize>
where
    T: Soa<'a> + ?Sized,
{
    type Refs<'ctx> = Slices<'ctx, 'a, T>;

    #[inline]
    fn get<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_ptrs(self, context, slices)?;
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_ptrs(self, context, slices);
        unsafe { context.slice_ptrs_to_slices(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_mut_ptrs(self, context, slices)?;
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_mut_ptrs(self, context, slices);
        unsafe { context.mut_slice_ptrs_to_mut_slices(slices) }
    }
}

unsafe impl<T> SoaSlicePtrsIndex<T> for ops::RangeFrom<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'ctx> = SlicePtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let Self { start } = self;
        let len = context.slice_ptrs_len(&slices);
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(start..len, context, slices) }
    }

    #[inline]
    fn get_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>> {
        let Self { start } = self;
        let len = context.slice_ptrs_len(&slices);
        SoaSlicePtrsIndex::<T>::get_ptrs(start..len, context, slices)
    }

    #[inline]
    fn index_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let Self { start } = self;
        let len = context.slice_ptrs_len(&slices);
        if start > len {
            slice_start_index_len_fail(start, len);
        }

        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(self, context, slices) }
    }

    type MutPtrs<'ctx> = SliceMutPtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let Self { start } = self;
        let len = context.mut_slice_ptrs_len(&slices);
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(start..len, context, slices) }
    }

    #[inline]
    fn get_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>> {
        let Self { start } = self;
        let len = context.mut_slice_ptrs_len(&slices);
        SoaSlicePtrsIndex::<T>::get_mut_ptrs(start..len, context, slices)
    }

    #[inline]
    fn index_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let Self { start } = self;
        let len = context.mut_slice_ptrs_len(&slices);
        if start > len {
            slice_start_index_len_fail(start, len);
        }

        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(self, context, slices) }
    }
}

unsafe impl<'a, T> SoaSlicesIndex<'a, T> for ops::RangeFrom<usize>
where
    T: Soa<'a> + ?Sized,
{
    type Refs<'ctx> = Slices<'ctx, 'a, T>;

    #[inline]
    fn get<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_ptrs(self, context, slices)?;
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_ptrs(self, context, slices);
        unsafe { context.slice_ptrs_to_slices(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_mut_ptrs(self, context, slices)?;
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_mut_ptrs(self, context, slices);
        unsafe { context.mut_slice_ptrs_to_mut_slices(slices) }
    }
}

unsafe impl<T> SoaSlicePtrsIndex<T> for ops::RangeFull
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'ctx> = SlicePtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked<'ctx>(
        self,
        _context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        slices
    }

    #[inline]
    fn get_ptrs<'ctx>(
        self,
        _context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>> {
        Some(slices)
    }

    #[inline]
    fn index_ptrs<'ctx>(
        self,
        _context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        slices
    }

    type MutPtrs<'ctx> = SliceMutPtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'ctx>(
        self,
        _context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        slices
    }

    #[inline]
    fn get_mut_ptrs<'ctx>(
        self,
        _context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>> {
        Some(slices)
    }

    #[inline]
    fn index_mut_ptrs<'ctx>(
        self,
        _context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        slices
    }
}

unsafe impl<'a, T> SoaSlicesIndex<'a, T> for ops::RangeFull
where
    T: Soa<'a> + ?Sized,
{
    type Refs<'ctx> = Slices<'ctx, 'a, T>;

    #[inline]
    fn get<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_ptrs(self, context, slices)?;
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_ptrs(self, context, slices);
        unsafe { context.slice_ptrs_to_slices(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_mut_ptrs(self, context, slices)?;
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_mut_ptrs(self, context, slices);
        unsafe { context.mut_slice_ptrs_to_mut_slices(slices) }
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

unsafe impl<T> SoaSlicePtrsIndex<T> for ops::RangeInclusive<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'ctx> = SlicePtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let range = range_into_slice_range(self);
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(range, context, slices) }
    }

    #[inline]
    fn get_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>> {
        if *self.end() == usize::MAX {
            return None;
        }
        let range = range_into_slice_range(self);
        SoaSlicePtrsIndex::<T>::get_ptrs(range, context, slices)
    }

    #[inline]
    fn index_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        let range = range_into_slice_range(self);
        SoaSlicePtrsIndex::<T>::index_ptrs(range, context, slices)
    }

    type MutPtrs<'ctx> = SliceMutPtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let range = range_into_slice_range(self);
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(range, context, slices) }
    }

    #[inline]
    fn get_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>> {
        if *self.end() == usize::MAX {
            return None;
        }
        let range = range_into_slice_range(self);
        SoaSlicePtrsIndex::<T>::get_mut_ptrs(range, context, slices)
    }

    #[inline]
    fn index_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        if *self.end() == usize::MAX {
            slice_end_index_overflow_fail();
        }
        let range = range_into_slice_range(self);
        SoaSlicePtrsIndex::<T>::index_mut_ptrs(range, context, slices)
    }
}

unsafe impl<'a, T> SoaSlicesIndex<'a, T> for ops::RangeInclusive<usize>
where
    T: Soa<'a> + ?Sized,
{
    type Refs<'ctx> = Slices<'ctx, 'a, T>;

    #[inline]
    fn get<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_ptrs(self, context, slices)?;
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_ptrs(self, context, slices);
        unsafe { context.slice_ptrs_to_slices(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_mut_ptrs(self, context, slices)?;
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_mut_ptrs(self, context, slices);
        unsafe { context.mut_slice_ptrs_to_mut_slices(slices) }
    }
}

unsafe impl<T> SoaSlicePtrsIndex<T> for ops::RangeToInclusive<usize>
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'ctx> = SlicePtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let Self { end } = self;
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(0..=end, context, slices) }
    }

    #[inline]
    fn get_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>> {
        let Self { end } = self;
        SoaSlicePtrsIndex::<T>::get_ptrs(0..=end, context, slices)
    }

    #[inline]
    fn index_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let Self { end } = self;
        SoaSlicePtrsIndex::<T>::index_ptrs(0..=end, context, slices)
    }

    type MutPtrs<'ctx> = SliceMutPtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let Self { end } = self;
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(0..=end, context, slices) }
    }

    #[inline]
    fn get_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>> {
        let Self { end } = self;
        SoaSlicePtrsIndex::<T>::get_mut_ptrs(0..=end, context, slices)
    }

    #[inline]
    fn index_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let Self { end } = self;
        SoaSlicePtrsIndex::<T>::index_mut_ptrs(0..=end, context, slices)
    }
}

unsafe impl<'a, T> SoaSlicesIndex<'a, T> for ops::RangeToInclusive<usize>
where
    T: Soa<'a> + ?Sized,
{
    type Refs<'ctx> = Slices<'ctx, 'a, T>;

    #[inline]
    fn get<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_ptrs(self, context, slices)?;
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_ptrs(self, context, slices);
        unsafe { context.slice_ptrs_to_slices(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_mut_ptrs(self, context, slices)?;
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_mut_ptrs(self, context, slices);
        unsafe { context.mut_slice_ptrs_to_mut_slices(slices) }
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

unsafe impl<T> SoaSlicePtrsIndex<T> for (ops::Bound<usize>, ops::Bound<usize>)
where
    T: RawSoa + ?Sized,
{
    type Ptrs<'ctx> = SlicePtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let len = context.slice_ptrs_len(&slices);
        let range = into_range_unchecked(len, self);
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(range, context, slices) }
    }

    #[inline]
    fn get_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>> {
        let len = context.slice_ptrs_len(&slices);
        let range = into_range(len, self)?;
        SoaSlicePtrsIndex::<T>::get_ptrs(range, context, slices)
    }

    #[inline]
    fn index_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let len = context.slice_ptrs_len(&slices);
        let range = into_slice_range(len, self);
        SoaSlicePtrsIndex::<T>::index_ptrs(range, context, slices)
    }

    type MutPtrs<'ctx> = SliceMutPtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let len = context.mut_slice_ptrs_len(&slices);
        let range = into_range_unchecked(len, self);
        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(range, context, slices) }
    }

    #[inline]
    fn get_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>> {
        let len = context.mut_slice_ptrs_len(&slices);
        let range = into_range(len, self)?;
        SoaSlicePtrsIndex::<T>::get_mut_ptrs(range, context, slices)
    }

    #[inline]
    fn index_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let len = context.mut_slice_ptrs_len(&slices);
        let range = into_slice_range(len, self);
        SoaSlicePtrsIndex::<T>::index_mut_ptrs(range, context, slices)
    }
}

unsafe impl<'a, T> SoaSlicesIndex<'a, T> for (ops::Bound<usize>, ops::Bound<usize>)
where
    T: Soa<'a> + ?Sized,
{
    type Refs<'ctx> = Slices<'ctx, 'a, T>;

    #[inline]
    fn get<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_ptrs(self, context, slices)?;
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_ptrs(self, context, slices);
        unsafe { context.slice_ptrs_to_slices(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::get_mut_ptrs(self, context, slices)?;
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = SoaSlicePtrsIndex::<T>::index_mut_ptrs(self, context, slices);
        unsafe { context.mut_slice_ptrs_to_mut_slices(slices) }
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

pub trait IndexHelper<'ctx, 'a, T>: SoaSlicesIndex<'a, T, Refs<'ctx> = &'a Self::Output>
where
    T: Soa<'a> + ?Sized,
{
    type Output: ?Sized + 'a;
}

impl<'ctx, 'a, T, I, U> IndexHelper<'ctx, 'a, T> for I
where
    U: ?Sized + 'a,
    T: Soa<'a> + ?Sized,
    I: SoaSlicesIndex<'a, T, Refs<'ctx> = &'a U>,
{
    type Output = U;
}

pub trait IndexHelperMut<'ctx, 'a, T>:
    IndexHelper<'ctx, 'a, T> + SoaSlicesIndex<'a, T, RefsMut<'ctx> = &'a mut Self::Output>
where
    T: Soa<'a> + ?Sized,
{
}

impl<'ctx, 'a, T, I, U> IndexHelperMut<'ctx, 'a, T> for I
where
    U: ?Sized + 'a,
    T: Soa<'a> + ?Sized,
    I: IndexHelper<'ctx, 'a, T, Output = U> + SoaSlicesIndex<'a, T, RefsMut<'ctx> = &'a mut U>,
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

#[cold]
#[inline(never)]
#[track_caller]
fn slice_index_usize_fail(len: usize, index: usize) -> ! {
    panic!("index out of bounds: the len is {len} but the index is {index}")
}

#[cold]
#[inline(never)]
#[track_caller]
fn slice_index_order_fail(index: usize, end: usize) -> ! {
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
fn slice_end_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range end index {index} out of range for slice of length {len}");
}

#[cold]
#[inline(never)]
#[track_caller]
const fn slice_end_index_overflow_fail() -> ! {
    panic!("attempted to index slice up to maximum usize");
}

#[cold]
#[inline(never)]
#[track_caller]
const fn slice_start_index_overflow_fail() -> ! {
    panic!("attempted to index slice from after maximum usize");
}
