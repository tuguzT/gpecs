pub use self::{
    mut_ptrs::{ErasedBundleMutPtrs, ErasedBundleMutPtrsIter},
    mut_refs::{ErasedBundleMutRefs, ErasedBundleMutRefsIter},
    mut_slice_ptrs::{ErasedBundleMutSlicePtrs, ErasedBundleMutSlicePtrsIter},
    mut_slices::{ErasedBundleMutSlices, ErasedBundleMutSlicesIter},
    nonnull_ptrs::{ErasedBundleNonNullPtrs, ErasedBundleNonNullPtrsIter},
    ptrs::{ErasedBundlePtrs, ErasedBundlePtrsIter},
    refs::{ErasedBundleRefs, ErasedBundleRefsIter},
    slice_ptrs::{ErasedBundleSlicePtrs, ErasedBundleSlicePtrsIter},
    slices::{ErasedBundleSlices, ErasedBundleSlicesIter},
    value::{
        ErasedBorrowedViewBundle, ErasedBorrowedViewBundleIntoIter, ErasedBundleIntoIterKind,
        ErasedBundleKind, ShuffledBundle,
    },
};

#[cfg(feature = "alloc")]
pub use crate::alloc::bundle::erased::{
    from_components::FromErasedComponent,
    remove::RemovePair,
    value::{
        ErasedBorrowedBundle, ErasedBorrowedBundleIntoIter, ErasedBundle, ErasedBundleIntoIter,
    },
};

pub mod error;
pub mod traits;

mod mut_ptrs;
mod mut_refs;
mod mut_slice_ptrs;
mod mut_slices;
mod nonnull_ptrs;
mod ptrs;
mod refs;
mod slice_ptrs;
mod slices;
mod soa_impl;
mod value;
