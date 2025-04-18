#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct WorldId(u16);

impl WorldId {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn into_u16(self) -> u16 {
        let Self(id) = self;
        id
    }

    #[inline]
    #[allow(unsafe_code)]
    pub const unsafe fn from_u16(id: u16) -> Self {
        Self(id)
    }
}

impl From<WorldId> for u16 {
    #[inline]
    fn from(value: WorldId) -> Self {
        value.into_u16()
    }
}
