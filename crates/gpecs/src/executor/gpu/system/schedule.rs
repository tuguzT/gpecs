use std::iter::{self, FusedIterator};

use indexmap::{IndexSet, set};

use super::registry::GpuSystemId;

#[derive(Debug, Default)]
pub struct GpuSystemSchedule {
    systems: IndexSet<GpuSystemId>,
}

impl GpuSystemSchedule {
    #[inline]
    pub fn new() -> Self {
        Self {
            systems: IndexSet::new(),
        }
    }

    #[inline]
    pub fn add_system(&mut self, system: GpuSystemId) -> bool {
        let Self { systems } = self;
        systems.insert(system)
    }

    #[inline]
    pub fn remove_system(&mut self, system: GpuSystemId) -> bool {
        let Self { systems } = self;
        systems.shift_remove(&system)
    }

    #[inline]
    pub fn iter(&self) -> GpuSystemScheduleIter<'_> {
        let Self { systems } = self;

        let inner = systems.iter().copied();
        GpuSystemScheduleIter { inner }
    }
}

impl<'a> IntoIterator for &'a GpuSystemSchedule {
    type Item = GpuSystemId;
    type IntoIter = GpuSystemScheduleIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct GpuSystemScheduleIter<'a> {
    inner: iter::Copied<set::Iter<'a, GpuSystemId>>,
}

impl Iterator for GpuSystemScheduleIter<'_> {
    type Item = GpuSystemId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next()
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
        inner.nth(n)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last()
    }

    #[inline]
    fn collect<B>(self) -> B
    where
        B: FromIterator<Self::Item>,
    {
        let Self { inner } = self;
        inner.collect()
    }
}

impl DoubleEndedIterator for GpuSystemScheduleIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n)
    }
}

impl ExactSizeIterator for GpuSystemScheduleIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for GpuSystemScheduleIter<'_> {}
