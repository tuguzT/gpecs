pub use self::{
    mut_ptrs::ErasedFieldMutPtr,
    mut_refs::ErasedFieldRefMut,
    mut_slice_ptrs::{ErasedFieldSliceMutPtr, field_slice_from_raw_parts_mut},
    mut_slices::ErasedFieldSliceMut,
    nonnull_ptrs::ErasedFieldNonNullPtr,
    ptrs::ErasedFieldPtr,
    refs::ErasedFieldRef,
    slice_ptrs::{ErasedFieldSlicePtr, field_slice_from_raw_parts},
    slices::ErasedFieldSlice,
    value::ErasedField,
};

#[cfg(feature = "alloc")]
pub use self::value::BoxedErasedField;

pub mod error;

mod assert;
mod mut_ptrs;
mod mut_refs;
mod mut_slice_ptrs;
mod mut_slices;
mod nonnull_ptrs;
mod ptrs;
mod refs;
mod slice_ptrs;
mod slices;
mod value;
