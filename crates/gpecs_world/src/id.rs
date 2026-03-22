use core::{
    error::Error,
    fmt::{self, Display},
};

use gpecs_num::u16::{U16FromU32Error, U16InU32};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct WorldId(U16InU32);

impl WorldId {
    #[inline]
    pub const fn new() -> Self {
        Self(U16InU32::MIN)
    }

    #[inline]
    pub const fn into_u16(self) -> u16 {
        let Self(id) = self;
        id.into_u16()
    }

    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(id) = self;
        id.into_u32()
    }

    #[inline]
    pub const unsafe fn from_u16(id: u16) -> Self {
        let id = U16InU32::from_u16(id);
        Self(id)
    }

    #[inline]
    pub const unsafe fn try_from_u32(id: u32) -> Result<Self, WorldIdFromU32Error> {
        match U16InU32::try_from_u32(id) {
            Ok(id) => Ok(Self(id)),
            Err(error) => Err(WorldIdFromU32Error(error)),
        }
    }

    #[inline]
    pub const unsafe fn from_u32(id: u32) -> Self {
        let id = unsafe { U16InU32::from_u32(id) };
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

impl Display for WorldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(id) = self;
        write!(f, "world {id}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct WorldIdFromU32Error(U16FromU32Error);

impl Display for WorldIdFromU32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(error) = self;
        write!(f, "`WorldId` {error}")
    }
}

impl Error for WorldIdFromU32Error {}
