use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_sparse::key::{Epoch, Key};

use crate::world::WorldId;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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
        let epoch_world = (epoch.into_u32() << Self::BITS) | world.into_u32();
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
    #[allow(unsafe_code)]
    pub const fn epoch(&self) -> EntityEpoch {
        let Self { epoch_world, .. } = *self;
        let epoch = epoch_world >> Self::BITS;
        unsafe { EntityEpoch::from_u32(epoch) }
    }

    #[inline]
    pub const fn set_epoch(&mut self, epoch: EntityEpoch) {
        let world = self.world().into_u32();
        self.epoch_world = (epoch.into_u32() << Self::BITS) | world;
    }

    #[inline]
    #[allow(unsafe_code)]
    pub const fn world(&self) -> WorldId {
        let Self { epoch_world, .. } = *self;
        let world = epoch_world & Self::LO_BITS_MASK;
        unsafe { WorldId::from_u32(world) }
    }

    #[inline]
    pub const fn set_world(&mut self, world: WorldId) {
        self.epoch_world = (self.epoch_world & Self::HI_BITS_MASK) | world.into_u32();
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index = self.index();
        let epoch = self.epoch().into_u32();
        let world = self.world().into_u32();

        f.debug_struct("Entity")
            .field("index", &index)
            .field("epoch", &epoch)
            .field("world", &world)
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

    fn new(sparse_index: Self::SparseIndex, epoch: Self::Epoch) -> Self {
        Entity::new(sparse_index, epoch, WorldId::default())
    }

    fn sparse_index(self) -> Self::SparseIndex {
        Entity::index(&self)
    }

    fn epoch(self) -> Self::Epoch {
        Entity::epoch(&self)
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct EntityEpoch(u32);

impl EntityEpoch {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn into_u16(self) -> u16 {
        let Self(epoch) = self;
        epoch as u16
    }

    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(epoch) = self;
        epoch
    }

    #[inline]
    pub const fn from_u16(epoch: u16) -> Self {
        Self(epoch as u32)
    }

    #[inline]
    pub const fn try_from_u32(epoch: u32) -> Result<Self, EpochFromU32Error> {
        const MAX: u32 = u16::MAX as u32;

        if epoch > MAX {
            Err(EpochFromU32Error)
        } else {
            Ok(Self(epoch))
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub const unsafe fn from_u32(epoch: u32) -> Self {
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
    fn from(value: EntityEpoch) -> Self {
        value.into_u16()
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
    fn from(value: EntityEpoch) -> Self {
        value.into_u32()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct EpochFromU32Error;

impl Display for EpochFromU32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to convert `u32` into `Epoch`")
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
