pub use self::{
    after::ArchetypesAfter,
    after_mut::ArchetypesAfterMut,
    before::ArchetypesBefore,
    before_mut::ArchetypesBeforeMut,
    bundles::{Bundles, BundlesIntoIter},
    bundles_mut::{BundlesMut, BundlesMutIntoIter},
    compatible::CompatibleArchetypes,
    compatible_mut::CompatibleArchetypesMut,
    cow::ErasedArchetypeCow,
    id::ArchetypeId,
    ids::ArchetypeIds,
    info::ArchetypeInfo,
    iter::Iter,
    iter_mut::IterMut,
    location::EntityLocation,
    registry::ArchetypeRegistry,
};

pub mod error;

mod after;
mod after_mut;
mod algo;
mod before;
mod before_mut;
mod bundles;
mod bundles_mut;
mod compatible;
mod compatible_mut;
mod cow;
mod id;
mod ids;
mod info;
mod iter;
mod iter_mut;
mod key;
mod location;
mod registry;
