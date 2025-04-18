use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    num::NonZeroU16,
    ops::Range,
};

pub use gpecs_types::world::WorldId;

#[derive(Debug, Clone)]
pub struct WorldRegistry {
    next_id: u16,
    len: NonZeroU16,
}

impl WorldRegistry {
    #[inline]
    pub const fn new() -> Self {
        Self {
            next_id: 1,
            len: NonZeroU16::MIN,
        }
    }

    #[inline]
    pub const fn spawn(&mut self) -> WorldId {
        let Self { next_id, len } = self;

        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        *len = len.saturating_add(1);
        world_id_trusted(id)
    }

    #[inline]
    pub const fn len(&self) -> u16 {
        let Self { len, .. } = *self;
        len.get()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn world_ids(&self) -> WorldIds {
        let len = self.len();
        WorldIds { inner: 0..len }
    }
}

impl Default for WorldRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WorldIds {
    inner: Range<u16>,
}

impl WorldIds {
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

impl Debug for WorldIds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let Range { start, end } = *inner;
        let inner = world_id_trusted(start)..world_id_trusted(end);
        write!(f, "{inner:?}")
    }
}

impl Iterator for WorldIds {
    type Item = WorldId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(world_id_trusted)
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
        inner.nth(n).map(world_id_trusted)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(world_id_trusted)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(world_id_trusted)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(world_id_trusted)
    }

    #[inline]
    fn is_sorted(self) -> bool {
        let Self { inner } = self;
        inner.is_sorted()
    }
}

impl DoubleEndedIterator for WorldIds {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(world_id_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(world_id_trusted)
    }
}

impl ExactSizeIterator for WorldIds {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for WorldIds {}

#[inline]
#[allow(unsafe_code)]
const fn world_id_trusted(id: u16) -> WorldId {
    unsafe { WorldId::from_inner(id) }
}
