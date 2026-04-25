pub use self::{
    component_ids::ComponentIds, iter::Iter, ordered_iter::ComponentIdOrderedIter,
    view::ErasedArchetypeView,
};

#[cfg(feature = "alloc")]
pub use crate::alloc::erased::{ErasedArchetype, FromComponentDescriptor, IntoIter};

pub mod error;

mod component_ids;
mod iter;
mod ordered_iter;
mod view;
