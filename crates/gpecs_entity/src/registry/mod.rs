pub use self::{
    iter::Iter, iter_mut::IterMut, view::EntityRegistryView, view_mut::EntityRegistryViewMut,
};

#[cfg(feature = "alloc")]
pub use self::alloc::{EntityRegistry, TryReserveError, TrySpawnError};

mod iter;
mod iter_mut;
mod view;
mod view_mut;

#[cfg(feature = "alloc")]
mod alloc;
