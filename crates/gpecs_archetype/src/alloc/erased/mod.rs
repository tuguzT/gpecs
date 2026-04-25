pub use self::{
    archetype::{ErasedArchetype, FromComponentDescriptor},
    into_iter::IntoIter,
};

pub mod error;

mod archetype;
mod into_iter;
mod view;
