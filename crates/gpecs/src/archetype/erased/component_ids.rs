use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    slice::Iter,
};

use crate::component::registry::ComponentId;

#[derive(Clone)]
#[repr(transparent)]
pub struct ComponentIds<'a> {
    inner: Iter<'a, u32>,
}

impl<'a> ComponentIds<'a> {
    #[inline]
    pub(super) fn from_inner(inner: Iter<'a, u32>) -> Self {
        Self { inner }
    }
}

impl Debug for ComponentIds<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl Iterator for ComponentIds<'_> {
    type Item = ComponentId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|&id| unsafe { ComponentId::from_u32(id) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|&id| unsafe { ComponentId::from_u32(id) })
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|&id| unsafe { ComponentId::from_u32(id) })
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner
            .map(|&id| unsafe { ComponentId::from_u32(id) })
            .collect()
    }
}

impl DoubleEndedIterator for ComponentIds<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|&id| unsafe { ComponentId::from_u32(id) })
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|&id| unsafe { ComponentId::from_u32(id) })
    }
}

impl ExactSizeIterator for ComponentIds<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for ComponentIds<'_> {}
