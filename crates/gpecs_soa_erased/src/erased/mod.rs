pub use self::{
    context::ErasedSoaContext,
    fields::ErasedSoaFields,
    nonnull_ptrs::{ErasedSoaNonNullPtrs, ErasedSoaNonNullPtrsIter},
    ptrs::{ErasedSoaPtrs, ErasedSoaPtrsIter},
    ptrs_mut::{ErasedSoaMutPtrs, ErasedSoaMutPtrsIter},
    refs::{ErasedSoaRefs, ErasedSoaRefsIter},
    refs_mut::{ErasedSoaRefsMut, ErasedSoaRefsMutIter},
    slice_ptrs::{ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter, slice_from_raw_parts},
    slice_ptrs_mut::{ErasedSoaSliceMutPtrs, ErasedSoaSliceMutPtrsIter, slice_from_raw_parts_mut},
    slices::{ErasedSoaSlices, ErasedSoaSlicesIter},
    slices_mut::{ErasedSoaSlicesMut, ErasedSoaSlicesMutIter},
    value::{ErasedSoa, ErasedSoaIntoFields},
};

#[cfg(feature = "alloc")]
pub use self::{context::BoxedErasedSoaContext, value::BoxedErasedSoa};

pub mod error;

mod assert;
mod context;
mod fields;
mod nonnull_ptrs;
mod ptrs;
mod ptrs_mut;
mod refs;
mod refs_mut;
mod slice_ptrs;
mod slice_ptrs_mut;
mod slices;
mod slices_mut;
mod soa_impl;
mod value;
