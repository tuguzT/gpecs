pub use self::{
    id::GpuComponentId, ids::GpuComponentIds, info::GpuComponentInfo,
    registry::GpuComponentRegistry,
};

mod descriptor;
mod id;
mod ids;
mod info;
mod registry;
