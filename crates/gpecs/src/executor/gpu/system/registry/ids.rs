use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::Range,
};

use crate::executor::gpu::system::registry::{GpuSystemId, id::gpu_system_id_trusted};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GpuSystemIds {
    inner: Range<u32>,
}

impl GpuSystemIds {
    #[inline]
    pub(super) fn new(inner: Range<u32>) -> Self {
        Self { inner }
    }
}

impl Debug for GpuSystemIds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let Range { start, end } = *inner;
        let ids = gpu_system_id_trusted(start)..gpu_system_id_trusted(end);
        f.debug_struct("GpuSystemIds").field("ids", &ids).finish()
    }
}

impl Iterator for GpuSystemIds {
    type Item = GpuSystemId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(gpu_system_id_trusted)
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
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(gpu_system_id_trusted)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(gpu_system_id_trusted)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(gpu_system_id_trusted)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(gpu_system_id_trusted)
    }

    #[inline]
    fn is_sorted(self) -> bool {
        let Self { inner } = self;
        inner.is_sorted()
    }
}

impl DoubleEndedIterator for GpuSystemIds {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(gpu_system_id_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(gpu_system_id_trusted)
    }
}

impl ExactSizeIterator for GpuSystemIds {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for GpuSystemIds {}
