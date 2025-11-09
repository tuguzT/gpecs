use core::fmt::{self, Debug, Display};

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
    fn from(id: ArchetypeId) -> Self {
        id.into_u32()
    }
}

impl Display for ArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(id) = self;
        write!(f, "archetype {id}")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct GpuArchetypeId(ArchetypeId);

impl GpuArchetypeId {
    #[inline]
    pub const fn into_id(self) -> ArchetypeId {
        let Self(id) = self;
        id
    }

    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(id) = self;
        id.into_u32()
    }

    #[inline]
    pub const unsafe fn from_id(id: ArchetypeId) -> Self {
        Self(id)
    }

    #[inline]
    pub const unsafe fn from_u32(id: u32) -> Self {
        let id = unsafe { ArchetypeId::from_u32(id) };
        Self(id)
    }
}

impl From<GpuArchetypeId> for u32 {
    #[inline]
    fn from(id: GpuArchetypeId) -> Self {
        id.into_u32()
    }
}

impl From<GpuArchetypeId> for ArchetypeId {
    #[inline]
    fn from(id: GpuArchetypeId) -> Self {
        id.into_id()
    }
}

impl Debug for GpuArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = &self.into_u32();
        f.debug_tuple("GpuArchetypeId").field(id).finish()
    }
}

impl Display for GpuArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(id) = self;

        if !f.alternate() {
            write!(f, "GPU ")?;
        }
        Display::fmt(id, f)
    }
}
