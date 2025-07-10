pub use self::{
    nonnull_ptrs::ErasedFieldNonNullPtr,
    ptrs::ErasedFieldPtr,
    ptrs_mut::ErasedFieldMutPtr,
    refs::ErasedFieldRef,
    refs_mut::ErasedFieldRefMut,
    slice_ptrs::{ErasedFieldSlicePtr, ErasedFieldSlicePtrIter, field_slice_from_raw_parts},
    slice_ptrs_mut::{
        ErasedFieldSliceMutPtr, ErasedFieldSliceMutPtrIter, field_slice_from_raw_parts_mut,
    },
    slices::{ErasedFieldSlice, ErasedFieldSliceIter},
    slices_mut::{ErasedFieldSliceIterMut, ErasedFieldSliceMut},
    value::ErasedField,
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
