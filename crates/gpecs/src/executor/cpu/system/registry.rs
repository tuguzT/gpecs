use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::Range,
};

use super::{IntoSystem, System};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct SystemId(usize);

impl SystemId {
    #[inline]
    pub const fn index(&self) -> usize {
        let Self(id) = *self;
        id
    }
}

#[derive(Debug)]
pub struct SystemInfo {
    id: SystemId,
    system: Box<dyn System>,
}

impl SystemInfo {
    #[inline]
    pub fn id(&self) -> SystemId {
        let Self { id, .. } = *self;
        id
    }

    #[inline]
    pub fn system(&self) -> &dyn System {
        let Self { system, .. } = self;
        system.as_ref()
    }

    #[inline]
    pub fn system_mut(&mut self) -> &mut dyn System {
        let Self { system, .. } = self;
        system.as_mut()
    }
}

#[derive(Debug, Default)]
pub struct SystemRegistry {
    systems: Vec<SystemInfo>,
}

impl SystemRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    #[inline]
    pub fn register_system<S, I>(&mut self, system: S) -> SystemId
    where
        S: IntoSystem<I>,
    {
        let Self { systems } = self;

        let id = SystemId(systems.len());

        let system = Box::new(system.into_system());
        let info = SystemInfo { id, system };
        systems.push(info);

        id
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { systems } = self;
        systems.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { systems } = self;
        systems.is_empty()
    }

    #[inline]
    pub fn get_system_info(&self, id: SystemId) -> Option<&SystemInfo> {
        let Self { systems } = self;
        systems.get(id.index())
    }

    #[inline]
    pub fn get_system_info_mut(&mut self, id: SystemId) -> Option<&mut SystemInfo> {
        let Self { systems } = self;
        systems.get_mut(id.index())
    }

    #[inline]
    pub fn system_ids(&self) -> SystemIds {
        let len = self.len();
        SystemIds { inner: 0..len }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SystemIds {
    inner: Range<usize>,
}

impl SystemIds {
    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { inner } = self;
        inner.is_empty()
    }
}

impl Debug for SystemIds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let Range { start, end } = *inner;
        let inner = SystemId(start)..SystemId(end);
        write!(f, "{inner:?}")
    }
}

impl Iterator for SystemIds {
    type Item = SystemId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(SystemId)
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
        inner.nth(n).map(SystemId)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(SystemId)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(SystemId)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(SystemId)
    }

    #[inline]
    fn is_sorted(self) -> bool {
        let Self { inner } = self;
        inner.is_sorted()
    }
}

impl DoubleEndedIterator for SystemIds {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(SystemId)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(SystemId)
    }
}

impl ExactSizeIterator for SystemIds {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for SystemIds {}
