pub use self::{
    iter::Iter, iter_mut::IterMut, view::EntityRegistryView, view_mut::EntityRegistryViewMut,
};

#[cfg(feature = "alloc")]
pub use self::alloc::{EntityRegistry, TryReserveError, TrySpawnError};

#[cfg(feature = "rayon")]
pub use self::{par_iter::ParIter, par_iter_mut::ParIterMut};

mod iter;
mod iter_mut;
mod view;
mod view_mut;

#[cfg(feature = "alloc")]
mod alloc;

#[cfg(feature = "rayon")]
mod par_iter;
#[cfg(feature = "rayon")]
mod par_iter_mut;
