use std::mem::MaybeUninit;

pub use gpecs_archetype::bundle::erased::*;

pub type ErasedBundleRefs<'a, D> =
    gpecs_archetype::bundle::erased::ErasedBundleRefs<'a, D, *const MaybeUninit<u8>>;
pub type ErasedBundleMutRefs<'a, D> =
    gpecs_archetype::bundle::erased::ErasedBundleMutRefs<'a, D, *mut MaybeUninit<u8>>;

pub type ErasedBundleSlices<'a, D> =
    gpecs_archetype::bundle::erased::ErasedBundleSlices<'a, D, *const MaybeUninit<u8>>;
pub type ErasedBundleMutSlices<'a, D> =
    gpecs_archetype::bundle::erased::ErasedBundleMutSlices<'a, D, *mut MaybeUninit<u8>>;

pub use self::value::{
    ErasedBorrowedBundle, ErasedBorrowedBundleIntoIter, ErasedBorrowedViewBundle,
    ErasedBorrowedViewBundleIntoIter, ErasedBundle, ErasedBundleIntoIter, ErasedBundleIntoIterKind,
    ErasedBundleKind, FromErasedComponent, RemovePair, ShuffledBundle,
};

pub mod error;

mod soa_impl;
mod value;
