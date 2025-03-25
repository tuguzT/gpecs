use std::{
    fmt::{self, Display},
    num::Wrapping,
};

use gpecs_sparse::key::{EpochKey, Key};

pub mod registry;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Entity {
    inner: EpochKey<u32, Wrapping<u16>>,
}

impl Entity {
    #[inline]
    pub const fn new(sparse_index: u32, epoch: u16) -> Self {
        let inner = EpochKey::new(sparse_index, Wrapping(epoch));
        Self { inner }
    }

    #[inline]
    pub const fn sparse_index(&self) -> u32 {
        let Self { inner } = self;
        *inner.sparse_index()
    }

    #[inline]
    pub const fn sparse_index_mut(&mut self) -> &mut u32 {
        let Self { inner } = self;
        inner.sparse_index_mut()
    }

    #[inline]
    pub const fn epoch(&self) -> u16 {
        let Self { inner } = self;
        inner.epoch().0
    }

    #[inline]
    pub const fn epoch_mut(&mut self) -> &mut u16 {
        let Self { inner } = self;
        &mut inner.epoch_mut().0
    }
}

impl Key for Entity {
    type SparseIndex = u32;
    type Epoch = Wrapping<u16>;

    fn new(sparse_index: Self::SparseIndex, epoch: Self::Epoch) -> Self {
        Entity::new(sparse_index, epoch.0)
    }

    fn sparse_index(self) -> Self::SparseIndex {
        Entity::sparse_index(&self)
    }

    fn epoch(self) -> Self::Epoch {
        Wrapping(Entity::epoch(&self))
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        write!(f, "{}v{}", inner.sparse_index(), inner.epoch())
    }
}
