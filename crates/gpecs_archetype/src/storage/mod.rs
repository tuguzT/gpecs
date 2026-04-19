pub use self::{
    entity::NoEpochEntity, traits::ErasedArchetypeSoa, view::ArchetypeStorageView,
    view_mut::ArchetypeStorageViewMut,
};

#[cfg(feature = "alloc")]
pub use crate::alloc::storage::ArchetypeStorage;

pub mod error;

mod entity;
mod traits;
mod view;
mod view_mut;
