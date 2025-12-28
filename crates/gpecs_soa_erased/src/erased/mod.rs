pub use self::{
    context::ErasedSoaContext,
    fields::ErasedSoaFields,
    mut_ptrs::{ErasedSoaMutPtrs, ErasedSoaMutPtrsIter},
    mut_refs::{ErasedSoaRefsMut, ErasedSoaRefsMutIter},
    mut_slice_ptrs::{ErasedSoaSliceMutPtrs, ErasedSoaSliceMutPtrsIter, slice_from_raw_parts_mut},
    mut_slices::{ErasedSoaSlicesMut, ErasedSoaSlicesMutIter},
    nonnull_ptrs::{ErasedSoaNonNullPtrs, ErasedSoaNonNullPtrsIter},
    ptrs::{ErasedSoaPtrs, ErasedSoaPtrsIter},
    refs::{ErasedSoaRefs, ErasedSoaRefsIter},
    slice_ptrs::{ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter, slice_from_raw_parts},
    slices::{ErasedSoaSlices, ErasedSoaSlicesIter},
    value::{ErasedSoa, ErasedSoaIntoFields},
};

#[cfg(feature = "alloc")]
pub use self::{context::BoxedErasedSoaContext, value::BoxedErasedSoa};

pub mod error;

mod assert;
mod context;
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
