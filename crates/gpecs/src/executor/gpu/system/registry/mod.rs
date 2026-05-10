pub use self::{
    id::GpuSystemId,
    ids::GpuSystemIds,
    registry::{GpuComponentAccess, GpuSystemDescriptor, GpuSystemRegistry},
};

mod id;
mod ids;
mod registry;
