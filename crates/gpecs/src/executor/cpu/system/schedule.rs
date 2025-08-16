use std::iter::{self, FusedIterator};

use indexmap::{IndexSet, set};

use super::registry::SystemId;

#[derive(Debug, Default)]
pub struct SystemSchedule {
    systems: IndexSet<SystemId>,
}

impl SystemSchedule {
    #[inline]
    pub fn new() -> Self {
        Self {
            systems: IndexSet::new(),
        }
    }

    #[inline]
    pub fn add_system(&mut self, system: SystemId) -> bool {
        let Self { systems } = self;
        systems.insert(system)
    }

    #[inline]
    pub fn remove_system(&mut self, system: SystemId) -> bool {
        let Self { systems } = self;
        systems.shift_remove(&system)
    }

    #[inline]
    pub fn iter(&self) -> SystemScheduleIter<'_> {
        let Self { systems } = self;

        let inner = systems.iter().copied();
        SystemScheduleIter { inner }
    }
}

impl<'a> IntoIterator for &'a SystemSchedule {
    type Item = SystemId;
    type IntoIter = SystemScheduleIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct SystemScheduleIter<'a> {
    inner: iter::Copied<set::Iter<'a, SystemId>>,
}

impl Iterator for SystemScheduleIter<'_> {
    type Item = SystemId;

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

impl DoubleEndedIterator for SystemScheduleIter<'_> {
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

impl ExactSizeIterator for SystemScheduleIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for SystemScheduleIter<'_> {}
