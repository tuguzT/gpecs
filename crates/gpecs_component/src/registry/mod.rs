pub use self::{
    id::{ComponentId, GpuComponentId},
    ids::ComponentIds,
    info::ComponentInfo,
    view::ComponentRegistryView,
};

#[cfg(feature = "alloc")]
pub use crate::alloc::{ComponentIdMap, ComponentRegistry};

pub mod traits;

mod id;
mod ids;
mod info;
mod view;

#[inline]
pub(crate) fn component_id_from_usize(index: usize) -> ComponentId {
    let id = index.try_into().expect("`ComponentId` overflow");
    component_id_trusted(id)
}

#[inline]
pub(crate) fn component_id_trusted(id: u32) -> ComponentId {
    unsafe { ComponentId::from_u32(id) }
}
