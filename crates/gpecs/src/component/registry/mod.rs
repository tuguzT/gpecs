pub use gpecs_component::registry::ComponentId;

pub use self::{
    alloc::ComponentRegistry, ids::ComponentIds, info::ComponentInfo, view::ComponentRegistryView,
};

pub mod traits;

mod alloc;
mod ids;
mod info;
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
