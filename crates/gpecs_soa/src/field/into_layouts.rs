use core::{alloc::Layout, iter::FusedIterator};

use crate::layout::WithLayout;

pub trait IntoFieldLayoutsIter: IntoIterator + Sized {
    #[inline]
    fn into_field_layouts(self) -> IntoFieldLayouts<Self::IntoIter> {
        self.into_iter().into()
    }
}

impl<T> IntoFieldLayoutsIter for T where T: IntoIterator {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct IntoFieldLayouts<T>(pub T)
where
    T: ?Sized;

impl<T> IntoFieldLayouts<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        let Self(inner) = self;
        inner
    }
}

impl<T> IntoFieldLayouts<T>
where
    T: ?Sized,
{
    #[inline]
    pub const fn as_inner(&self) -> &T {
        let Self(inner) = self;
        inner
    }

    #[inline]
    pub const fn as_mut_inner(&mut self) -> &mut T {
        let Self(inner) = self;
        inner
    }
}

impl<T> From<T> for IntoFieldLayouts<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Iterator for IntoFieldLayouts<T>
where
    T: Iterator<Item: WithLayout> + ?Sized,
{
    type Item = Layout;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self(inner) = self;
        inner.next().map(|item| item.layout())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self(inner) = self;
        inner.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self(inner) = self;
        inner.nth(n).map(|item| item.layout())
    }
}

impl<T> DoubleEndedIterator for IntoFieldLayouts<T>
where
    T: DoubleEndedIterator<Item: WithLayout> + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self(inner) = self;
        inner.next_back().map(|item| item.layout())
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self(inner) = self;
        inner.nth_back(n).map(|item| item.layout())
    }
}

impl<T> ExactSizeIterator for IntoFieldLayouts<T>
where
    T: ExactSizeIterator<Item: WithLayout> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self(inner) = self;
        inner.len()
    }
}

impl<T> FusedIterator for IntoFieldLayouts<T> where T: FusedIterator<Item: WithLayout> + ?Sized {}
