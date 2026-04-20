use core::slice;

use crate::soa::{
    field::{FieldLayoutsOutput, FieldLayoutsOwned},
    identity::Identity,
    layout::WithLayout,
};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

pub trait CovariantFieldLayouts<T = Self>: FieldLayoutsOwned<T>
where
    T: ?Sized,
{
    /// Restricts [field layouts](crate::soa::field::FieldLayouts::Output)
    /// to be covariant over generic lifetime.
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self, T>,
    ) -> FieldLayoutsOutput<'short, Self, T>;
}

impl<'a, T, U> CovariantFieldLayouts<&'a U> for &'a T
where
    T: CovariantFieldLayouts<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self, &'a U>,
    ) -> FieldLayoutsOutput<'short, Self, &'a U> {
        T::upcast_field_layouts(from)
    }
}

#[cfg(feature = "alloc")]
impl<T, U> CovariantFieldLayouts<Box<U>> for Box<T>
where
    T: CovariantFieldLayouts<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self, Box<U>>,
    ) -> FieldLayoutsOutput<'short, Self, Box<U>> {
        T::upcast_field_layouts(from)
    }
}

impl<T, U> CovariantFieldLayouts<Identity<U>> for Identity<T>
where
    T: CovariantFieldLayouts<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self, Identity<U>>,
    ) -> FieldLayoutsOutput<'short, Self, Identity<U>> {
        T::upcast_field_layouts(from)
    }
}

impl<T> CovariantFieldLayouts for [T]
where
    T: WithLayout + 'static,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}

impl<T, const N: usize> CovariantFieldLayouts for [T; N]
where
    T: WithLayout + 'static,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}

impl<T> CovariantFieldLayouts for slice::Iter<'_, T>
where
    T: WithLayout + 'static,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}
