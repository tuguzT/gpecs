use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    slice::Iter,
};

use crate::{archetype::erased::ErasedArchetype, component::registry::ComponentId};

#[derive(Clone)]
#[repr(transparent)]
pub struct ErasedArchetypeComponentIds<'a> {
    inner: Iter<'a, u32>,
}

impl<'a> ErasedArchetypeComponentIds<'a> {
    #[inline]
    pub fn new(archetype: &'a ErasedArchetype<impl Sized>) -> Self {
        let inner = archetype.components.as_key_slice().iter();
        Self { inner }
    }
}

impl Debug for ErasedArchetypeComponentIds<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl Iterator for ErasedArchetypeComponentIds<'_> {
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

impl DoubleEndedIterator for ErasedArchetypeComponentIds<'_> {
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

impl ExactSizeIterator for ErasedArchetypeComponentIds<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for ErasedArchetypeComponentIds<'_> {}
