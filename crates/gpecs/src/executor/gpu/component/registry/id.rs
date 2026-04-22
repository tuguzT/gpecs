use crate::component::registry::ComponentId;

pub use gpecs_component::registry::GpuComponentId;

#[inline]
pub fn gpu_component_id_trusted(id: ComponentId) -> GpuComponentId {
    unsafe { GpuComponentId::from_id(id) }
}

#[inline]
pub fn gpu_component_id_u32_trusted(id: u32) -> GpuComponentId {
    unsafe { GpuComponentId::from_u32(id) }
}
