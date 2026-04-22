use crate::archetype::registry::ArchetypeId;

pub use gpecs_archetype::registry::GpuArchetypeId;

#[inline]
pub fn gpu_archetype_id_trusted(id: ArchetypeId) -> GpuArchetypeId {
    unsafe { GpuArchetypeId::from_id(id) }
}

#[inline]
pub fn gpu_archetype_id_u32_trusted(id: u32) -> GpuArchetypeId {
    unsafe { GpuArchetypeId::from_u32(id) }
}
