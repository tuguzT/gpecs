pub use self::{
    id::GpuSystemId,
    ids::GpuSystemIds,
    info::GpuSystemInfo,
    registry::{
        DEFAULT_WORKGROUP_SIZE, DispatchStrategy, GpuComponentAccess, GpuSystemDescriptor,
        GpuSystemRegistry,
    },
};

mod id;
mod ids;
mod info;
mod registry;
