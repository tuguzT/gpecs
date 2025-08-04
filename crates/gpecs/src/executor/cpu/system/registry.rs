use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::Range,
};

use super::{IntoSystem, System};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct SystemId(u32);

impl SystemId {
    #[inline]
    pub const fn into_u32(&self) -> u32 {
        let Self(id) = *self;
        id
    }

    #[inline]
    pub const unsafe fn from_u32(id: u32) -> Self {
        Self(id)
    }
}

impl From<SystemId> for u32 {
    #[inline]
    fn from(id: SystemId) -> Self {
        id.into_u32()
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
    pub fn register_system<S, In>(&mut self, system: S) -> SystemId
    where
        S: IntoSystem<In>,
    {
        let Self { systems } = self;

        let index = systems.len();
        let id = system_id_from_usize(index);

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
        systems.get(system_id_into_usize(id))
    }

    #[inline]
    pub fn get_system_info_mut(&mut self, id: SystemId) -> Option<&mut SystemInfo> {
        let Self { systems } = self;
        systems.get_mut(system_id_into_usize(id))
    }

    #[inline]
    pub fn system_ids(&self) -> SystemIds {
        let index = self.len();
        let len = system_id_from_usize(index).into_u32();
        SystemIds { inner: 0..len }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SystemIds {
    inner: Range<u32>,
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
        let ids = system_id_trusted(start)..system_id_trusted(end);
        f.debug_struct("SystemIds").field("ids", &ids).finish()
    }
}

impl Iterator for SystemIds {
    type Item = SystemId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(system_id_trusted)
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
        inner.nth(n).map(system_id_trusted)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(system_id_trusted)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(system_id_trusted)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(system_id_trusted)
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
        inner.next_back().map(system_id_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(system_id_trusted)
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

#[inline]
fn system_id_from_usize(index: usize) -> SystemId {
    let id = index.try_into().expect("`SystemId` overflow");
    system_id_trusted(id)
}

#[inline]
fn system_id_into_usize(id: SystemId) -> usize {
    let id = id.into_u32();
    id.try_into().expect("`SystemId` overflow")
}

#[inline]
fn system_id_trusted(id: u32) -> SystemId {
    unsafe { SystemId::from_u32(id) }
}
