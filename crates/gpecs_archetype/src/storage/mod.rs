pub use self::{
    bundle_iter::BundleIter, bundle_iter_mut::BundleIterMut, entity::NoEpochEntity, iter::Iter,
    iter_mut::IterMut, traits::ErasedArchetypeSoa, view::ArchetypeStorageView,
    view_mut::ArchetypeStorageViewMut,
};

#[cfg(feature = "alloc")]
pub use crate::alloc::storage::ArchetypeStorage;

#[cfg(feature = "rayon")]
pub use self::{bundle_par_iter::BundleParIter, bundle_par_iter_mut::BundleParIterMut};

pub mod error;

mod bundle_iter;
mod bundle_iter_mut;
mod entity;
mod iter;
mod iter_mut;
mod traits;
mod view;
mod view_mut;

#[cfg(feature = "rayon")]
mod bundle_par_iter;
#[cfg(feature = "rayon")]
mod bundle_par_iter_mut;
