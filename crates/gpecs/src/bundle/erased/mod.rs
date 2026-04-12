use std::{mem::MaybeUninit, ptr::NonNull};

pub use gpecs_archetype::bundle::erased::*;

pub type ErasedBundlePtrs<D> =
    gpecs_archetype::bundle::erased::ErasedBundlePtrs<D, *const MaybeUninit<u8>>;
pub type ErasedBundleMutPtrs<D> =
    gpecs_archetype::bundle::erased::ErasedBundleMutPtrs<D, *mut MaybeUninit<u8>>;
pub type ErasedBundleNonNullPtrs<D> =
    gpecs_archetype::bundle::erased::ErasedBundleNonNullPtrs<D, NonNull<MaybeUninit<u8>>>;

pub type ErasedBundleRefs<'a, D> =
    gpecs_archetype::bundle::erased::ErasedBundleRefs<'a, D, *const MaybeUninit<u8>>;
pub type ErasedBundleMutRefs<'a, D> =
    gpecs_archetype::bundle::erased::ErasedBundleMutRefs<'a, D, *mut MaybeUninit<u8>>;

pub type ErasedBundleRefsIter<'a, D> =
    gpecs_archetype::bundle::erased::ErasedBundleRefsIter<'a, D, *const MaybeUninit<u8>>;
pub type ErasedBundleMutRefsIter<'a, D> =
    gpecs_archetype::bundle::erased::ErasedBundleMutRefsIter<'a, D, *mut MaybeUninit<u8>>;

pub type ErasedBundleSlicePtrs<D> =
    gpecs_archetype::bundle::erased::ErasedBundleSlicePtrs<D, *const MaybeUninit<u8>>;
pub type ErasedBundleMutSlicePtrs<D> =
    gpecs_archetype::bundle::erased::ErasedBundleMutSlicePtrs<D, *mut MaybeUninit<u8>>;

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
