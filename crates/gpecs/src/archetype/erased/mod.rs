pub use self::{
    alloc::{ErasedArchetype, ErasedArchetypeIntoIter, FromComponentInfo},
    component_ids::ComponentIds,
    iter::Iter,
    ordered_iter::ComponentIdOrderedIter,
    view::ErasedArchetypeView,
};

pub mod error;

mod alloc;
mod component_ids;
mod iter;
mod ordered_iter;
mod view;
