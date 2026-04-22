use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    slice,
};

use super::{GpuComponentId, id::gpu_component_id_u32_trusted};

#[derive(Clone)]
pub struct GpuComponentIds<'a> {
    inner: slice::Iter<'a, u32>,
}

impl<'a> GpuComponentIds<'a> {
    #[inline]
    pub(super) fn new(inner: slice::Iter<'a, u32>) -> Self {
        Self { inner }
    }
}

impl Debug for GpuComponentIds<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl Iterator for GpuComponentIds<'_> {
    type Item = GpuComponentId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().copied().map(gpu_component_id_u32_trusted)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().copied().map(gpu_component_id_u32_trusted)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).copied().map(gpu_component_id_u32_trusted)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.copied().map(gpu_component_id_u32_trusted).for_each(f);
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_component_id_u32_trusted)
            .fold(init, f)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().map(gpu_component_id_u32_trusted).all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().map(gpu_component_id_u32_trusted).any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_component_id_u32_trusted)
            .find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.copied().map(gpu_component_id_u32_trusted).find_map(f)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_component_id_u32_trusted)
            .position(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_component_id_u32_trusted)
            .rposition(predicate)
    }
}

impl DoubleEndedIterator for GpuComponentIds<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().copied().map(gpu_component_id_u32_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).copied().map(gpu_component_id_u32_trusted)
    }
}

impl ExactSizeIterator for GpuComponentIds<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for GpuComponentIds<'_> {}
