pub use self::{
    archetype::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
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
        ErasedBorrowedBundle, ErasedBorrowedBundleIntoIter, ErasedBorrowedViewBundle,
        ErasedBorrowedViewBundleIntoIter, ErasedBundle, ErasedBundleIntoIter,
        ErasedBundleIntoIterKind, ErasedBundleKind, FromErasedComponent, RemovePair,
        ShuffledBundle,
    },
};

pub mod error;

mod archetype;
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
