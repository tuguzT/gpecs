pub use self::{
    archetype::{ErasedArchetype, FromComponentDescriptor},
    into_iter::IntoIter,
};

mod archetype;
mod into_iter;
mod view;
