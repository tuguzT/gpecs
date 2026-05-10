pub use self::{
    id::{ComponentId, GpuComponentId},
    ids::ComponentIds,
    registry::ComponentRegistry,
    view::ComponentRegistryView,
};

#[cfg(feature = "alloc")]
pub use crate::alloc::ComponentIdMap;

pub mod traits;

mod id;
mod ids;
mod registry;
mod view;

#[inline]
fn component_id_from_usize(index: usize) -> ComponentId {
    let id = index.try_into().expect("`ComponentId` overflow");
    component_id_trusted(id)
}

#[inline]
fn component_id_trusted(id: u32) -> ComponentId {
    unsafe { ComponentId::from_u32(id) }
}
