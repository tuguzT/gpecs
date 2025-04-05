use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::Range,
};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct WorldId(u16);

impl WorldId {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn index(&self) -> u16 {
        let Self(id) = *self;
        id
    }
}

impl From<WorldId> for u16 {
    #[inline]
    fn from(value: WorldId) -> Self {
        value.index()
    }
}

#[derive(Debug, Clone)]
pub struct WorldRegistry {
    next_id: u16,
    len: u16,
}

impl WorldRegistry {
    #[inline]
    pub const fn new() -> Self {
        Self { next_id: 1, len: 1 }
    }

    #[inline]
    pub const fn spawn(&mut self) -> WorldId {
        let Self { next_id, len } = self;

        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        *len = len.saturating_add(1);
        WorldId(id)
    }

    #[inline]
    pub const fn len(&self) -> u16 {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn world_ids(&self) -> WorldIds {
        let Self { len, .. } = *self;
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
        let inner = WorldId(start)..WorldId(end);
        write!(f, "{inner:?}")
    }
}

impl Iterator for WorldIds {
    type Item = WorldId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(WorldId)
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
        inner.nth(n).map(WorldId)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(WorldId)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(WorldId)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(WorldId)
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
        inner.next_back().map(WorldId)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(WorldId)
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
