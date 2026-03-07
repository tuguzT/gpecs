use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use bytemuck::{Pod, Zeroable};
use gpecs_sparse::key::{Epoch, Key};

use crate::world::WorldId;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Entity {
    index: u32,
    epoch_world: u32,
}

impl Entity {
    const BITS: u32 = u16::BITS;
    const LO_BITS_MASK: u32 = u16::MAX as u32;
    const HI_BITS_MASK: u32 = !Self::LO_BITS_MASK;

    #[inline]
    pub const fn new(index: u32, epoch: EntityEpoch, world: WorldId) -> Self {
        let epoch_world = (world.into_u32() << Self::BITS) | epoch.into_u32();
        Self { index, epoch_world }
    }

    #[inline]
    pub const fn index(&self) -> u32 {
        let Self { index, .. } = *self;
        index
    }

    #[inline]
    pub const fn set_index(&mut self, index: u32) {
        self.index = index;
    }

    #[inline]
    pub const fn epoch(&self) -> EntityEpoch {
        let Self { epoch_world, .. } = *self;
        let epoch = epoch_world & Self::LO_BITS_MASK;
        unsafe { EntityEpoch::from_u32(epoch) }
    }

    #[inline]
    pub const fn set_epoch(&mut self, epoch: EntityEpoch) {
        self.epoch_world = (self.epoch_world & Self::HI_BITS_MASK) | epoch.into_u32();
    }

    #[inline]
    pub const fn world(&self) -> WorldId {
        let Self { epoch_world, .. } = *self;
        let epoch = epoch_world >> Self::BITS;
        unsafe { WorldId::from_u32(epoch) }
    }

    #[inline]
    pub const fn set_world(&mut self, world: WorldId) {
        let epoch = Self::epoch(self);
        self.epoch_world = (world.into_u32() << Self::BITS) | epoch.into_u32();
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index = &self.index();
        let epoch = &self.epoch();
        let world = &self.world();

        f.debug_struct("Entity")
            .field("index", index)
            .field("epoch", epoch)
            .field("world", world)
            .finish()
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index = self.index();
        let epoch = self.epoch().into_u32();
        let world = self.world().into_u32();
        write!(f, "entity{{i{index}e{epoch}w{world}}}")
    }
}

impl Key for Entity {
    type SparseIndex = u32;
    type Epoch = EntityEpoch;

    #[inline]
    fn new(sparse_index: Self::SparseIndex, epoch: Self::Epoch) -> Self {
        Self::new(sparse_index, epoch, WorldId::default())
    }

    #[inline]
    fn sparse_index(self) -> Self::SparseIndex {
        Self::index(&self)
    }

    #[inline]
    fn epoch(self) -> Self::Epoch {
        Self::epoch(&self)
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct EntityEpoch(u32);

impl EntityEpoch {
    const MAX: u32 = u16::MAX as u32;

    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    pub const fn into_u16(self) -> u16 {
        let Self(epoch) = self;
        debug_assert!(epoch <= Self::MAX, "`EntityEpoch` should fit into `u16`");
        epoch as u16
    }

    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(epoch) = self;
        debug_assert!(epoch <= Self::MAX, "`EntityEpoch` should fit into `u16`");
        epoch
    }

    #[inline]
    pub const fn from_u16(epoch: u16) -> Self {
        Self(epoch as u32)
    }

    #[inline]
    pub const fn try_from_u32(epoch: u32) -> Result<Self, EpochFromU32Error> {
        if epoch > Self::MAX {
            Err(EpochFromU32Error)
        } else {
            Ok(Self(epoch))
        }
    }

    #[inline]
    pub const unsafe fn from_u32(epoch: u32) -> Self {
        debug_assert!(epoch <= Self::MAX, "`EntityEpoch` should fit into `u16`");
        Self(epoch)
    }
}

impl From<u16> for EntityEpoch {
    #[inline]
    fn from(value: u16) -> Self {
        Self::from_u16(value)
    }
}

impl From<EntityEpoch> for u16 {
    #[inline]
    fn from(epoch: EntityEpoch) -> Self {
        epoch.into_u16()
    }
}

impl TryFrom<u32> for EntityEpoch {
    type Error = EpochFromU32Error;

    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::try_from_u32(value)
    }
}

impl From<EntityEpoch> for u32 {
    #[inline]
    fn from(epoch: EntityEpoch) -> Self {
        epoch.into_u32()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct EpochFromU32Error;

impl Display for EpochFromU32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`EntityEpoch` should fit into `u16`")
    }
}

impl Error for EpochFromU32Error {}

impl Epoch for EntityEpoch {
    #[inline]
    fn next(self) -> Self {
        let epoch = self.into_u32() + 1;
        Self::try_from_u32(epoch).unwrap_or_default()
    }
}
