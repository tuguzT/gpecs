pub use self::{
    nonnull_ptrs::ErasedFieldNonNullPtr,
    ptrs::ErasedFieldPtr,
    ptrs_mut::ErasedFieldMutPtr,
    refs::ErasedFieldRef,
    refs_mut::ErasedFieldRefMut,
    slice_ptrs::{field_slice_from_raw_parts, ErasedFieldSlicePtr, ErasedFieldSlicePtrIter},
    slice_ptrs_mut::{
        field_slice_from_raw_parts_mut, ErasedFieldSliceMutPtr, ErasedFieldSliceMutPtrIter,
    },
    slices::{ErasedFieldSlice, ErasedFieldSliceIter},
    slices_mut::{ErasedFieldSliceIterMut, ErasedFieldSliceMut},
    value::ErasedField,
    vecs::ErasedFieldVec,
};

pub mod error;

mod assert;
mod nonnull_ptrs;
mod ptrs;
mod ptrs_mut;
mod refs;
mod refs_mut;
mod slice_ptrs;
mod slice_ptrs_mut;
mod slices;
mod slices_mut;
mod value;
mod vecs;
