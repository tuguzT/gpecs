use gpecs_archetype::id::{ArchetypeId, GpuArchetypeId};

#[test]
fn new() {
    let id = unsafe { ArchetypeId::from_u32(42) };
    assert_eq!(u32::from(id), 42);

    let gpu_id = unsafe { GpuArchetypeId::from_u32(42) };
    assert_eq!(u32::from(gpu_id), 42);
    assert_eq!(ArchetypeId::from(gpu_id), id);
    assert_eq!(unsafe { GpuArchetypeId::from_id(id) }, gpu_id);
}

#[test]
fn fmt() {
    let id = unsafe { ArchetypeId::from_u32(42) };
    assert_eq!(format!("{id:?}"), "ArchetypeId(42)");

    let gpu_id = unsafe { GpuArchetypeId::from_u32(42) };
    assert_eq!(format!("{gpu_id:?}"), "GpuArchetypeId(42)");
}
