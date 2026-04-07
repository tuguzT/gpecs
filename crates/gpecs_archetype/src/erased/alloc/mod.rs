pub use self::{
    archetype::{ErasedArchetype, FromComponentInfo},
    into_iter::IntoIter,
    view_ext::ErasedArchetypeViewExt,
};

pub mod error;

mod archetype;
mod into_iter;
mod view_ext;
