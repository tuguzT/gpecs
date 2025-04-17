use std::{
    fmt::{self, Display},
    num::Wrapping,
};

use gpecs_sparse::key::Key;

use crate::world::registry::WorldId;

pub mod registry;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(C)]
pub struct Entity {
    index: u32,
    epoch: u16,
    world: WorldId,
}

impl Entity {
    #[inline]
    pub const fn new(index: u32, epoch: u16, world: WorldId) -> Self {
        Self {
            index,
            epoch,
            world,
        }
    }

    #[inline]
    pub const fn index(&self) -> u32 {
        let Self { index, .. } = *self;
        index
    }

    #[inline]
    pub const fn index_mut(&mut self) -> &mut u32 {
        let Self { index, .. } = self;
        index
    }

    #[inline]
    pub const fn epoch(&self) -> u16 {
        let Self { epoch, .. } = *self;
        epoch
    }

    #[inline]
    pub const fn epoch_mut(&mut self) -> &mut u16 {
        let Self { epoch, .. } = self;
        epoch
    }

    #[inline]
    pub const fn world(&self) -> WorldId {
        let Self { world, .. } = *self;
        world
    }

    #[inline]
    pub const fn world_mut(&mut self) -> &mut WorldId {
        let Self { world, .. } = self;
        world
    }
}

impl Key for Entity {
    type SparseIndex = u32;
    type Epoch = Wrapping<u16>;

    fn new(sparse_index: Self::SparseIndex, epoch: Self::Epoch) -> Self {
        let Wrapping(epoch) = epoch;
        Entity::new(sparse_index, epoch, WorldId::default())
    }

    fn sparse_index(self) -> Self::SparseIndex {
        Entity::index(&self)
    }

    fn epoch(self) -> Self::Epoch {
        Wrapping(Entity::epoch(&self))
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            index,
            epoch,
            world,
        } = self;
        let world = world.index();
        write!(f, "{index}v{epoch}w{world}")
    }
}
