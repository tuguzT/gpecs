#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct ComponentId(u32);

impl ComponentId {
    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(id) = self;
        id
    }

    #[inline]
    #[allow(unsafe_code)]
    pub const unsafe fn from_u32(id: u32) -> Self {
        Self(id)
    }
}

impl From<ComponentId> for u32 {
    #[inline]
    fn from(value: ComponentId) -> Self {
        value.into_u32()
    }
}
