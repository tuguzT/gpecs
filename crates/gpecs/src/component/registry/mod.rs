pub use gpecs_component::id::ComponentId;

pub use self::{
    alloc::{ComponentIds, ComponentInfo, ComponentRegistry},
    descriptor::ErasedDropComponentDescriptor,
    mapping::ComponentTypeIdMap,
};

pub mod traits;

mod alloc;
mod descriptor;
mod mapping;
