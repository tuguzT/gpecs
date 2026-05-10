//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

pub use self::{
    field_layouts::KeyValueFieldLayouts, mut_ptrs::KeyValueMutPtrs, mut_refs::KeyValueMutRefs,
    mut_slice_ptrs::KeyValueMutSlicePtrs, mut_slices::KeyValueMutSlices,
    nonnull_ptrs::KeyValueNonNullPtrs, ptrs::KeyValuePtrs, refs::KeyValueRefs,
    slice_ptrs::KeyValueSlicePtrs, slices::KeyValueSlices, value::KeyValuePair,
};

mod field_layouts;
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
