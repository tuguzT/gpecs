pub use self::{
    id::GpuSystemId,
    ids::GpuSystemIds,
    info::GpuSystemInfo,
    registry::{GpuComponentAccess, GpuSystemDescriptor, GpuSystemRegistry},
};

mod id;
mod ids;
mod info;
mod registry;
