use gpecs_types::component::{ComponentId, GpuComponentId};

#[test]
fn new() {
    let id = unsafe { ComponentId::from_u32(42) };
    assert_eq!(u32::from(id), 42);

    let gpu_id = unsafe { GpuComponentId::from_u32(42) };
    assert_eq!(u32::from(gpu_id), 42);
    assert_eq!(ComponentId::from(gpu_id), id);
    assert_eq!(unsafe { GpuComponentId::from_id(id) }, gpu_id);
}

#[test]
fn fmt() {
    let id = unsafe { ComponentId::from_u32(42) };
    assert_eq!(format!("{id:?}"), "ComponentId(42)");

    let gpu_id = unsafe { GpuComponentId::from_u32(42) };
    assert_eq!(format!("{gpu_id:?}"), "GpuComponentId(42)");
}
