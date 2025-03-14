use std::fmt::{self, Display};

use gpecs_sparse::key::{EpochKey, Key};

pub mod registry;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Entity {
    inner: EpochKey,
}

impl Entity {
    #[inline]
    pub const fn new(sparse_index: usize, epoch: usize) -> Self {
        let inner = EpochKey::new(sparse_index, epoch);
        Self { inner }
    }

    #[inline]
    pub const fn sparse_index(&self) -> usize {
        let Self { inner } = self;
        inner.sparse_index()
    }

    #[inline]
    pub const fn sparse_index_mut(&mut self) -> &mut usize {
        let Self { inner } = self;
        inner.sparse_index_mut()
    }

    #[inline]
    pub const fn epoch(&self) -> usize {
        let Self { inner } = self;
        *inner.epoch()
    }

    #[inline]
    pub const fn epoch_mut(&mut self) -> &mut usize {
        let Self { inner } = self;
        inner.epoch_mut()
    }
}

impl Key for Entity {
    type Epoch = usize;

    fn new(sparse_index: usize, epoch: Self::Epoch) -> Self {
        Entity::new(sparse_index, epoch)
    }

    fn sparse_index(self) -> usize {
        Entity::sparse_index(&self)
    }

    fn epoch(self) -> Self::Epoch {
        Entity::epoch(&self)
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        inner.fmt(f)
    }
}
