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

    #[inline]
    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        unsafe { get_offset_unchecked::<T>(context, slices, self) }
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
        if self >= len {
            slice_index_usize_fail(len, self)
        }

        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(self, context, slices) }
    }

    type MutPtrs<'ctx> = MutPtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        unsafe { get_offset_unchecked_mut::<T>(context, slices, self) }
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
        if self >= len {
            slice_index_usize_fail(len, self)
        }

        unsafe { SoaSlicePtrsIndex::<T>::get_unchecked_mut(self, context, slices) }
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

    #[inline]
    unsafe fn get_unchecked<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
        let Self { start, end } = self;
        let new_len = unsafe { end.unchecked_sub(start) };
        unsafe { get_offset_len_unchecked::<T>(context, slices, start, new_len) }
    }

    #[inline]
    fn get_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Option<Self::Ptrs<'ctx>> {
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
    fn index_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicePtrs<'ctx, T>,
    ) -> Self::Ptrs<'ctx> {
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

    type MutPtrs<'ctx> = SliceMutPtrs<'ctx, T>;

    #[inline]
    unsafe fn get_unchecked_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
        let Self { start, end } = self;
        let new_len = unsafe { end.unchecked_sub(start) };
        unsafe { get_offset_len_unchecked_mut::<T>(context, slices, start, new_len) }
    }

    #[inline]
    fn get_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Option<Self::MutPtrs<'ctx>> {
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
    fn index_mut_ptrs<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SliceMutPtrs<'ctx, T>,
    ) -> Self::MutPtrs<'ctx> {
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
            slice_index_fail(start, len, len)
        }

        let new_len = unsafe { len.unchecked_sub(start) };
        unsafe { get_offset_len_unchecked::<T>(context, slices, start, new_len) }
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
            slice_index_fail(start, len, len)
        }

        let new_len = unsafe { len.unchecked_sub(start) };
        unsafe { get_offset_len_unchecked_mut::<T>(context, slices, start, new_len) }
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
        _context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Option<Self::Refs<'ctx>> {
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        _context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        slices
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        _context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        _context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
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
        let end = *self.end();
        let len = context.slice_ptrs_len(&slices);
        if end >= len {
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
        let start = *self.start();
        let end = *self.end();
        let len = context.slice_ptrs_len(&slices);
        if end >= len {
            slice_index_fail(start, end, len)
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
        let end = *self.end();
        let len = context.mut_slice_ptrs_len(&slices);
        if end >= len {
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
        let start = *self.start();
        let end = *self.end();
        let len = context.mut_slice_ptrs_len(&slices);
        if end >= len {
            slice_index_fail(start, end, len)
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
        let range = try_into_slice_range(len, self)?;
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
        let range = try_into_slice_range(len, self)?;
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

#[inline]
unsafe fn get_offset_unchecked<'ctx, T>(
    context: &'ctx T::Context,
    slices: SlicePtrs<'ctx, T>,
    offset: usize,
) -> Ptrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    let ptrs = context.slice_ptrs_as_ptrs(slices);
    unsafe { context.ptrs_add(ptrs, offset) }
}

#[inline]
unsafe fn get_offset_unchecked_mut<'ctx, T>(
    context: &'ctx T::Context,
    slices: SliceMutPtrs<'ctx, T>,
    offset: usize,
) -> MutPtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    let ptrs = context.mut_slice_ptrs_as_ptrs(slices);
    unsafe { context.ptrs_add_mut(ptrs, offset) }
}

/// Copy of private `core::slice::index::get_offset_len_noubcheck()`.
#[inline]
unsafe fn get_offset_len_unchecked<'ctx, T>(
    context: &'ctx T::Context,
    slices: SlicePtrs<'ctx, T>,
    offset: usize,
    len: usize,
) -> SlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    let data = unsafe { get_offset_unchecked::<T>(context, slices, offset) };
    context.slice_ptrs_from_raw_parts(data, len)
}

/// Copy of private `core::slice::index::get_offset_len_mut_noubcheck()`.
#[inline]
unsafe fn get_offset_len_unchecked_mut<'ctx, T>(
    context: &'ctx T::Context,
    slices: SliceMutPtrs<'ctx, T>,
    offset: usize,
    len: usize,
) -> SliceMutPtrs<'ctx, T>
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
