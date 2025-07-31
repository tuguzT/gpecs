pub use self::{
    context::{BoxedErasedSoaContext, ErasedSoaContext},
    fields::ErasedSoaFields,
    nonnull_ptrs::{ErasedSoaNonNullPtrs, ErasedSoaNonNullPtrsIter},
    ptrs::{ErasedSoaPtrs, ErasedSoaPtrsIter},
    ptrs_mut::{ErasedSoaMutPtrs, ErasedSoaMutPtrsIter},
    refs::{ErasedSoaRefs, ErasedSoaRefsIter},
    refs_mut::{ErasedSoaRefsMut, ErasedSoaRefsMutIter},
    slice_ptrs::{ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter, soa_slice_from_raw_parts},
    slice_ptrs_mut::{
        ErasedSoaSliceMutPtrs, ErasedSoaSliceMutPtrsIter, soa_slice_from_raw_parts_mut,
    },
    slices::{ErasedSoaSlices, ErasedSoaSlicesIter},
    slices_mut::{ErasedSoaSlicesMut, ErasedSoaSlicesMutIter},
    value::{BoxedErasedSoa, ErasedSoa, ErasedSoaIntoFields},
};

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
