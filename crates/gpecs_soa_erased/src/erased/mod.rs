pub use self::{
    context::ErasedSoaContext,
    fields::ErasedSoaFields,
    nonnull_ptrs::ErasedSoaNonNullPtrs,
    ptrs::ErasedSoaPtrs,
    ptrs_mut::ErasedSoaMutPtrs,
    refs::ErasedSoaRefs,
    refs_mut::ErasedSoaRefsMut,
    slice_ptrs::{ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter},
    slice_ptrs_mut::{ErasedSoaSliceMutPtrs, ErasedSoaSliceMutPtrsIter},
    slices::{ErasedSoaSlices, ErasedSoaSlicesIter},
    slices_mut::{ErasedSoaSlicesIterMut, ErasedSoaSlicesMut},
    value::ErasedSoa,
    vecs::ErasedSoaVecs,
};

pub mod error;

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
mod vecs;
