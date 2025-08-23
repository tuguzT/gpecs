use core::{
    error::Error,
    fmt::{self, Display},
};

use bytemuck::{Pod, Zeroable};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct WorldId(u32);

impl WorldId {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    pub const fn into_u16(self) -> u16 {
        let Self(id) = self;
        id as u16
    }

    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(id) = self;
        id
    }

    #[inline]
    pub const unsafe fn from_u16(id: u16) -> Self {
        Self(id as u32)
    }

    #[inline]
    pub unsafe fn try_from_u32(id: u32) -> Result<Self, WorldIdFromU32Error> {
        const MAX: u32 = u16::MAX as u32;

        if id > MAX {
            Err(WorldIdFromU32Error)
        } else {
            Ok(Self(id))
        }
    }

    #[inline]
    pub const unsafe fn from_u32(id: u32) -> Self {
        Self(id)
    }
}

impl From<WorldId> for u16 {
    #[inline]
    fn from(id: WorldId) -> Self {
        id.into_u16()
    }
}

impl From<WorldId> for u32 {
    #[inline]
    fn from(id: WorldId) -> Self {
        id.into_u32()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct WorldIdFromU32Error;

impl Display for WorldIdFromU32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to convert `u32` into `WorldId`")
    }
}

impl Error for WorldIdFromU32Error {}
