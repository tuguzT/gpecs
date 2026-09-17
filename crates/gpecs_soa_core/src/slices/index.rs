use core::ops;

use crate::{
    ptrs::{SlicePtrsIndex, get, get_mut, index, index_mut},
    traits::{Refs, RefsMut, Slices, SlicesMut, Soa, SoaContext},
};

pub unsafe trait SlicesIndex<'a, T>: SlicePtrsIndex<T>
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

unsafe impl<'a, T> SlicesIndex<'a, T> for usize
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
        let ptrs = get::<T, _>(context, slices, self)?;
        let refs = unsafe { context.refs_from_ptrs(ptrs) };
        Some(refs)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let ptrs = index::<T, _>(context, slices, self);
        unsafe { context.refs_from_ptrs(ptrs) }
    }

    type RefsMut<'ctx> = RefsMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let ptrs = get_mut::<T, _>(context, slices, self)?;
        let refs = unsafe { context.mut_refs_from_mut_ptrs(ptrs) };
        Some(refs)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let ptrs = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_refs_from_mut_ptrs(ptrs) }
    }
}

unsafe impl<'a, T> SlicesIndex<'a, T> for ops::Range<usize>
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
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'a, T> SlicesIndex<'a, T> for ops::RangeTo<usize>
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
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'a, T> SlicesIndex<'a, T> for ops::RangeFrom<usize>
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
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'a, T> SlicesIndex<'a, T> for ops::RangeFull
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

unsafe impl<'a, T> SlicesIndex<'a, T> for ops::RangeInclusive<usize>
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
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'a, T> SlicesIndex<'a, T> for ops::RangeToInclusive<usize>
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
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'a, T> SlicesIndex<'a, T> for (ops::Bound<usize>, ops::Bound<usize>)
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
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: Slices<'ctx, 'a, T>,
    ) -> Self::Refs<'ctx> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'ctx> = SlicesMut<'ctx, 'a, T>;

    #[inline]
    fn get_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Option<Self::RefsMut<'ctx>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'ctx>(
        self,
        context: &'ctx T::Context,
        slices: SlicesMut<'ctx, 'a, T>,
    ) -> Self::RefsMut<'ctx> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

#[doc(hidden)]
pub trait IndexHelper<'ctx, 'a, T>: SlicesIndex<'a, T, Refs<'ctx> = &'a Self::Output>
where
    T: Soa<'a> + ?Sized,
{
    type Output: ?Sized + 'a;
}

impl<'ctx, 'a, T, I, U> IndexHelper<'ctx, 'a, T> for I
where
    U: ?Sized + 'a,
    T: Soa<'a> + ?Sized,
    I: SlicesIndex<'a, T, Refs<'ctx> = &'a U>,
{
    type Output = U;
}

#[doc(hidden)]
pub trait IndexHelperMut<'ctx, 'a, T>:
    IndexHelper<'ctx, 'a, T> + SlicesIndex<'a, T, RefsMut<'ctx> = &'a mut Self::Output>
where
    T: Soa<'a> + ?Sized,
{
}

impl<'ctx, 'a, T, I, U> IndexHelperMut<'ctx, 'a, T> for I
where
    U: ?Sized + 'a,
    T: Soa<'a> + ?Sized,
    I: IndexHelper<'ctx, 'a, T, Output = U> + SlicesIndex<'a, T, RefsMut<'ctx> = &'a mut U>,
{
}
