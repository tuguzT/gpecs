//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use gpecs_erased::{data, layout, ptr, storage, uninit};
pub use gpecs_soa as soa;

pub use self::{
    context::ErasedSoaContext,
    descriptors::CovariantFieldDescriptors,
    fields::ErasedSoaFields,
    mut_ptrs::{ErasedSoaMutPtrs, ErasedSoaMutPtrsIter},
    mut_refs::{ErasedSoaMutRefs, ErasedSoaMutRefsIter},
    mut_slice_ptrs::{ErasedSoaMutSlicePtrs, ErasedSoaMutSlicePtrsIter},
    mut_slices::{ErasedSoaMutSlices, ErasedSoaMutSlicesIter},
    nonnull_ptrs::{ErasedSoaNonNullPtrs, ErasedSoaNonNullPtrsIter},
    ptrs::{ErasedSoaPtrs, ErasedSoaPtrsIter},
    refs::{ErasedSoaRefs, ErasedSoaRefsIter},
    slice_ptrs::{ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter},
    slices::{ErasedSoaSlices, ErasedSoaSlicesIter},
    value::{ErasedSoa, ErasedSoaIntoFields},
};

#[cfg(feature = "alloc")]
pub use self::{context::BoxedErasedSoaContext, value::BoxedErasedSoa};

pub mod error;

mod assert;
mod context;
mod dangling;
mod descriptors;
mod fields;
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
