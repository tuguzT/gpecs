use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use bytemuck::Contiguous;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct U16InU32(u32);

impl U16InU32 {
    const MIN_U32: u32 = u16::MIN as u32;
    const MAX_U32: u32 = u16::MAX as u32;

    pub const MIN: Self = Self::from_u16(u16::MIN);
    pub const MAX: Self = Self::from_u16(u16::MAX);

    #[inline]
    pub const fn from_u16(value: u16) -> Self {
        Self(value as u32)
    }

    #[inline]
    pub const unsafe fn from_u32(value: u32) -> Self {
        debug_assert!(value <= Self::MAX_U32, "value should fit into `u16`");
        Self(value)
    }

    #[inline]
    pub const fn try_from_u32(value: u32) -> Result<Self, U16FromU32Error> {
        if value > Self::MAX_U32 {
            return Err(U16FromU32Error);
        }

        let me = unsafe { Self::from_u32(value) };
        Ok(me)
    }

    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    pub const fn into_u16(self) -> u16 {
        let Self(value) = self;
        debug_assert!(value <= Self::MAX_U32, "value should fit into `u16`");
        value as u16
    }

    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(value) = self;
        debug_assert!(value <= Self::MAX_U32, "value should fit into `u16`");
        value
    }
}

impl From<U16InU32> for u16 {
    #[inline]
    fn from(value: U16InU32) -> Self {
        value.into_u16()
    }
}

impl From<U16InU32> for u32 {
    #[inline]
    fn from(value: U16InU32) -> Self {
        value.into_u32()
    }
}

impl From<u16> for U16InU32 {
    #[inline]
    fn from(value: u16) -> Self {
        Self::from_u16(value)
    }
}

impl TryFrom<u32> for U16InU32 {
    type Error = U16FromU32Error;

    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::try_from_u32(value)
    }
}

impl Debug for U16InU32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(value) = self;
        Debug::fmt(value, f)
    }
}

impl Display for U16InU32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(value) = self;
        Display::fmt(value, f)
    }
}

unsafe impl Contiguous for U16InU32 {
    type Int = u32;

    const MAX_VALUE: Self::Int = Self::MAX_U32;
    const MIN_VALUE: Self::Int = Self::MIN_U32;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct U16FromU32Error;

impl Display for U16FromU32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value should fit into `u16`")
    }
}

impl Error for U16FromU32Error {}
