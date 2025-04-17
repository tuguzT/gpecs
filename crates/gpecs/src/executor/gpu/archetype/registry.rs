use crate::archetype::registry::ArchetypeId;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct GpuArchetypeId(ArchetypeId);

impl GpuArchetypeId {
    #[inline]
    pub fn index(&self) -> usize {
        let Self(id) = *self;
        id.index()
    }

    #[inline]
    pub const fn into_inner(self) -> u32 {
        let Self(id) = self;
        id.into_inner()
    }
}

impl From<GpuArchetypeId> for ArchetypeId {
    #[inline]
    fn from(value: GpuArchetypeId) -> Self {
        let GpuArchetypeId(id) = value;
        id
    }
}
