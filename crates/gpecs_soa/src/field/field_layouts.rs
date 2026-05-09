use core::slice;

#[cfg(feature = "alloc")]
use core_alloc::boxed::Box;

use crate::layout::WithLayout;

/// Alias for the [field layouts](FieldLayouts::Output) of some type `T`.
pub type FieldLayoutsOutput<'a, T, U = T> = <T as FieldLayouts<'a, U>>::Output;

/// Alias for an iterator of the [field layouts](FieldLayouts::Output) of some type `T`.
pub type FieldLayoutsIter<'a, T, U = T> = <FieldLayoutsOutput<'a, T, U> as IntoIterator>::IntoIter;

/// Alias for an iterator item of the [field layouts](FieldLayouts::Output) of some type `T`.
pub type FieldLayoutsItem<'a, T, U = T> = <FieldLayoutsOutput<'a, T, U> as IntoIterator>::Item;

/// Used to retrieve a non-owning collection of field layouts.
pub trait FieldLayouts<'a, T = Self>
where
    T: ?Sized,
{
    /// Collection of items which could be converted into a [layout](core::alloc::Layout).
    type Output: IntoIterator<IntoIter = Self::OutputIter, Item = Self::OutputItem> + 'a;
    type OutputIter: Iterator<Item = Self::OutputItem>;
    type OutputItem: WithLayout;

    /// Returns [field layouts](FieldLayouts::Output) from self.
    fn field_layouts(&'a self) -> Self::Output;
}

impl<'a, T, U> FieldLayouts<'a, &U> for &T
where
    T: FieldLayouts<'a, U> + ?Sized,
    U: ?Sized,
{
    type Output = T::Output;
    type OutputIter = T::OutputIter;
    type OutputItem = T::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        (**self).field_layouts()
    }
}

#[cfg(feature = "alloc")]
impl<'a, T, U> FieldLayouts<'a, Box<U>> for Box<T>
where
    T: FieldLayouts<'a, U> + ?Sized,
    U: ?Sized,
{
    type Output = T::Output;
    type OutputIter = T::OutputIter;
    type OutputItem = T::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        (**self).field_layouts()
    }
}

impl<'a, T> FieldLayouts<'a> for [T]
where
    T: WithLayout + 'a,
{
    type Output = &'a [T];
    type OutputIter = slice::Iter<'a, T>;
    type OutputItem = &'a T;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self
    }
}

impl<'a, T, const N: usize> FieldLayouts<'a> for [T; N]
where
    T: WithLayout + 'a,
{
    type Output = &'a [T; N];
    type OutputIter = slice::Iter<'a, T>;
    type OutputItem = &'a T;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self
    }
}

impl<'a, T> FieldLayouts<'a> for slice::Iter<'_, T>
where
    T: WithLayout + 'a,
{
    type Output = slice::Iter<'a, T>;
    type OutputIter = slice::Iter<'a, T>;
    type OutputItem = &'a T;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self.clone()
    }
}

/// Used to retrieve a non-owning collection of field layouts for any lifetime.
pub trait FieldLayoutsOwned<T = Self>: for<'a> FieldLayouts<'a, T>
where
    T: ?Sized,
{
}

impl<T, U> FieldLayoutsOwned<U> for T
where
    T: for<'a> FieldLayouts<'a, U> + ?Sized,
    U: ?Sized,
{
}
