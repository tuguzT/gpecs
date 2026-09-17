use core::ops;

use crate::{
    ptrs::{SlicePtrsIndex, get, get_mut, index, index_mut},
    traits::{Refs, RefsMut, Slices, SlicesMut, Soa, SoaContext},
};

pub unsafe trait SlicesIndex<'data, T>: SlicePtrsIndex<T>
where
    T: Soa<'data> + ?Sized,
{
    type Refs<'a>;

    fn get<'a>(
        self,
        context: &'a T::Context,
        slices: Slices<'a, 'data, T>,
    ) -> Option<Self::Refs<'a>>;

    fn index<'a>(self, context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Self::Refs<'a>;

    type RefsMut<'a>;

    fn get_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Option<Self::RefsMut<'a>>;

    fn index_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Self::RefsMut<'a>;
}

unsafe impl<'data, T> SlicesIndex<'data, T> for usize
where
    T: Soa<'data> + ?Sized,
{
    type Refs<'a> = Refs<'a, 'data, T>;

    #[inline]
    fn get<'a>(
        self,
        context: &'a T::Context,
        slices: Slices<'a, 'data, T>,
    ) -> Option<Self::Refs<'a>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let ptrs = get::<T, _>(context, slices, self)?;
        let refs = unsafe { context.refs_from_ptrs(ptrs) };
        Some(refs)
    }

    #[inline]
    fn index<'a>(self, context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Self::Refs<'a> {
        let slices = context.slices_as_slice_ptrs(slices);
        let ptrs = index::<T, _>(context, slices, self);
        unsafe { context.refs_from_ptrs(ptrs) }
    }

    type RefsMut<'a> = RefsMut<'a, 'data, T>;

    #[inline]
    fn get_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Option<Self::RefsMut<'a>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let ptrs = get_mut::<T, _>(context, slices, self)?;
        let refs = unsafe { context.mut_refs_from_mut_ptrs(ptrs) };
        Some(refs)
    }

    #[inline]
    fn index_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Self::RefsMut<'a> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let ptrs = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_refs_from_mut_ptrs(ptrs) }
    }
}

unsafe impl<'data, T> SlicesIndex<'data, T> for ops::Range<usize>
where
    T: Soa<'data> + ?Sized,
{
    type Refs<'a> = Slices<'a, 'data, T>;

    #[inline]
    fn get<'a>(
        self,
        context: &'a T::Context,
        slices: Slices<'a, 'data, T>,
    ) -> Option<Self::Refs<'a>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'a>(self, context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Self::Refs<'a> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'a> = SlicesMut<'a, 'data, T>;

    #[inline]
    fn get_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Option<Self::RefsMut<'a>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Self::RefsMut<'a> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'data, T> SlicesIndex<'data, T> for ops::RangeTo<usize>
where
    T: Soa<'data> + ?Sized,
{
    type Refs<'a> = Slices<'a, 'data, T>;

    #[inline]
    fn get<'a>(
        self,
        context: &'a T::Context,
        slices: Slices<'a, 'data, T>,
    ) -> Option<Self::Refs<'a>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'a>(self, context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Self::Refs<'a> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'a> = SlicesMut<'a, 'data, T>;

    #[inline]
    fn get_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Option<Self::RefsMut<'a>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Self::RefsMut<'a> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'data, T> SlicesIndex<'data, T> for ops::RangeFrom<usize>
where
    T: Soa<'data> + ?Sized,
{
    type Refs<'a> = Slices<'a, 'data, T>;

    #[inline]
    fn get<'a>(
        self,
        context: &'a T::Context,
        slices: Slices<'a, 'data, T>,
    ) -> Option<Self::Refs<'a>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'a>(self, context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Self::Refs<'a> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'a> = SlicesMut<'a, 'data, T>;

    #[inline]
    fn get_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Option<Self::RefsMut<'a>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Self::RefsMut<'a> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'data, T> SlicesIndex<'data, T> for ops::RangeFull
where
    T: Soa<'data> + ?Sized,
{
    type Refs<'a> = Slices<'a, 'data, T>;

    #[inline]
    fn get<'a>(
        self,
        _context: &'a T::Context,
        slices: Slices<'a, 'data, T>,
    ) -> Option<Self::Refs<'a>> {
        Some(slices)
    }

    #[inline]
    fn index<'a>(self, _context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Self::Refs<'a> {
        slices
    }

    type RefsMut<'a> = SlicesMut<'a, 'data, T>;

    #[inline]
    fn get_mut<'a>(
        self,
        _context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Option<Self::RefsMut<'a>> {
        Some(slices)
    }

    #[inline]
    fn index_mut<'a>(
        self,
        _context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Self::RefsMut<'a> {
        slices
    }
}

unsafe impl<'data, T> SlicesIndex<'data, T> for ops::RangeInclusive<usize>
where
    T: Soa<'data> + ?Sized,
{
    type Refs<'a> = Slices<'a, 'data, T>;

    #[inline]
    fn get<'a>(
        self,
        context: &'a T::Context,
        slices: Slices<'a, 'data, T>,
    ) -> Option<Self::Refs<'a>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'a>(self, context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Self::Refs<'a> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'a> = SlicesMut<'a, 'data, T>;

    #[inline]
    fn get_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Option<Self::RefsMut<'a>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Self::RefsMut<'a> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'data, T> SlicesIndex<'data, T> for ops::RangeToInclusive<usize>
where
    T: Soa<'data> + ?Sized,
{
    type Refs<'a> = Slices<'a, 'data, T>;

    #[inline]
    fn get<'a>(
        self,
        context: &'a T::Context,
        slices: Slices<'a, 'data, T>,
    ) -> Option<Self::Refs<'a>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'a>(self, context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Self::Refs<'a> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'a> = SlicesMut<'a, 'data, T>;

    #[inline]
    fn get_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Option<Self::RefsMut<'a>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Self::RefsMut<'a> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = index_mut::<T, _>(context, slices, self);
        unsafe { context.mut_slices_from_mut_slice_ptrs(slices) }
    }
}

unsafe impl<'data, T> SlicesIndex<'data, T> for (ops::Bound<usize>, ops::Bound<usize>)
where
    T: Soa<'data> + ?Sized,
{
    type Refs<'a> = Slices<'a, 'data, T>;

    #[inline]
    fn get<'a>(
        self,
        context: &'a T::Context,
        slices: Slices<'a, 'data, T>,
    ) -> Option<Self::Refs<'a>> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = get::<T, _>(context, slices, self)?;
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index<'a>(self, context: &'a T::Context, slices: Slices<'a, 'data, T>) -> Self::Refs<'a> {
        let slices = context.slices_as_slice_ptrs(slices);
        let slices = index::<T, _>(context, slices, self);
        unsafe { context.slices_from_slice_ptrs(slices) }
    }

    type RefsMut<'a> = SlicesMut<'a, 'data, T>;

    #[inline]
    fn get_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Option<Self::RefsMut<'a>> {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        let slices = get_mut::<T, _>(context, slices, self)?;
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        Some(slices)
    }

    #[inline]
    fn index_mut<'a>(
        self,
        context: &'a T::Context,
        slices: SlicesMut<'a, 'data, T>,
    ) -> Self::RefsMut<'a> {
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
