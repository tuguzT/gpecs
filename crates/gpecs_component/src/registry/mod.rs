pub use self::{
    id::{ComponentId, GpuComponentId},
    ids::ComponentIds,
    info::ComponentInfo,
    view::ComponentRegistryView,
};

#[cfg(feature = "alloc")]
pub use self::alloc::ComponentRegistry;

pub mod traits;

mod id;
mod ids;
mod info;
mod view;

#[cfg(feature = "alloc")]
mod alloc;
