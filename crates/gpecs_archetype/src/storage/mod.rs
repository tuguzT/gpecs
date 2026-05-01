pub use self::{
    entity::NoEpochEntity, iter::Iter, iter_mut::IterMut, traits::ErasedArchetypeSoa,
    view::ArchetypeStorageView, view_mut::ArchetypeStorageViewMut,
};

#[cfg(feature = "alloc")]
pub use crate::alloc::storage::ArchetypeStorage;

pub mod error;

mod entity;
mod iter;
mod iter_mut;
mod traits;
mod view;
mod view_mut;
