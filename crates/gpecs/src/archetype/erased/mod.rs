pub use self::{
    alloc::{ErasedArchetype, ErasedArchetypeIntoIter, FromComponentInfo},
    component_ids::ErasedArchetypeComponentIds,
    iter::ErasedArchetypeIter,
    sorted_iter::ErasedArchetypeSortedIter,
    view::ErasedArchetypeView,
};

mod alloc;
mod component_ids;
mod iter;
mod sorted_iter;
mod view;
