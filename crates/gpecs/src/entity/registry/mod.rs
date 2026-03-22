pub use self::{
    alloc::{EntityRegistry, TryReserveError, TrySpawnError},
    iter::Iter,
    iter_mut::IterMut,
    view::EntityRegistryView,
};

mod alloc;
mod iter;
mod iter_mut;
mod view;
mod view_mut;
