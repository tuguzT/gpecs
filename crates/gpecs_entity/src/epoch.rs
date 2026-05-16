use core::{
    error::Error,
    fmt::{self, Display},
};

use bytemuck::{Pod, Zeroable};
use gpecs_num::u16::{U16FromU32Error, U16InU32};
use gpecs_sparse::key::Epoch;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct EntityEpoch(U16InU32);

impl EntityEpoch {
    #[inline]
    pub const fn new() -> Self {
        Self(U16InU32::MIN)
    }

    #[inline]
    pub const fn into_u16(self) -> u16 {
        let Self(epoch) = self;
        epoch.into_u16()
    }

    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(epoch) = self;
        epoch.into_u32()
    }

    #[inline]
    pub const fn from_u16(epoch: u16) -> Self {
        let epoch = U16InU32::from_u16(epoch);
        Self(epoch)
    }

    #[inline]
    pub const fn try_from_u32(epoch: u32) -> Result<Self, EpochFromU32Error> {
        match U16InU32::try_from_u32(epoch) {
            Ok(epoch) => Ok(Self(epoch)),
            Err(error) => Err(EpochFromU32Error(error)),
        }
    }

    #[inline]
    pub const unsafe fn from_u32(epoch: u32) -> Self {
        let id = unsafe { U16InU32::from_u32(epoch) };
        Self(id)
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
pub struct EpochFromU32Error(U16FromU32Error);

impl Display for EpochFromU32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(error) = self;
        write!(f, "`EntityEpoch` {error}")
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
