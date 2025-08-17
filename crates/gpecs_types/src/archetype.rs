use bytemuck::{Pod, Zeroable};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(id) = self;
        id
    }

    #[inline]
    pub const unsafe fn from_u32(id: u32) -> Self {
        Self(id)
    }
}

impl From<ArchetypeId> for u32 {
    #[inline]
    fn from(value: ArchetypeId) -> Self {
        value.into_u32()
    }
}
