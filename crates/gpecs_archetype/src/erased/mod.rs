pub use self::{
    component_ids::ComponentIds, iter::Iter, ordered_iter::ComponentIdOrderedIter,
    view::ErasedArchetypeView,
};

#[cfg(feature = "alloc")]
pub use self::alloc::{ErasedArchetype, ErasedArchetypeIntoIter, FromComponentInfo};

pub mod error;

mod component_ids;
mod iter;
mod ordered_iter;
mod view;

#[cfg(feature = "alloc")]
mod alloc;
