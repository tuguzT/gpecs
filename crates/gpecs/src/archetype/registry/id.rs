pub use gpecs_archetype::registry::ArchetypeId;

#[inline]
pub fn archetype_id_from_usize(index: usize) -> ArchetypeId {
    let id = index.try_into().expect("`ArchetypeId` overflow");
    archetype_id_trusted(id)
}

#[inline]
pub fn archetype_id_into_usize(id: ArchetypeId) -> usize {
    let id = id.into_u32();
    id.try_into().expect("`ArchetypeId` overflow")
}

#[inline]
pub fn archetype_id_trusted(id: u32) -> ArchetypeId {
    unsafe { ArchetypeId::from_u32(id) }
}
