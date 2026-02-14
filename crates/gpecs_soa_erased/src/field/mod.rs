pub use self::{
    mut_ptr::ErasedFieldMutPtr, mut_ref::ErasedFieldRefMut, mut_slice::ErasedFieldSliceMut,
    mut_slice_ptr::ErasedFieldSliceMutPtr, nonnull_ptr::ErasedFieldNonNullPtr, ptr::ErasedFieldPtr,
    r#ref::ErasedFieldRef, slice::ErasedFieldSlice, slice_ptr::ErasedFieldSlicePtr,
    value::ErasedField,
};

#[cfg(feature = "alloc")]
pub use self::value::BoxedErasedField;

pub mod error;

mod mut_ptr;
mod mut_ref;
mod mut_slice;
mod mut_slice_ptr;
mod nonnull_ptr;
mod ptr;
mod r#ref;
mod slice;
mod slice_ptr;
mod value;
