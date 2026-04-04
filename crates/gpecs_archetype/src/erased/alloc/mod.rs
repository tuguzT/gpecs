pub use self::{
    archetype::{ErasedArchetype, FromComponentInfo},
    into_iter::ErasedArchetypeIntoIter,
};

pub mod error;

mod archetype;
mod into_iter;
