pub use self::{
    mut_ptr::ErasedMutPtr,
    mut_ref::ErasedMutRef,
    mut_slice::ErasedMutSlice,
    mut_slice_ptr::ErasedMutSlicePtr,
    nonnull_ptr::ErasedNonNullPtr,
    ptr::ErasedPtr,
    r#ref::ErasedRef,
    slice::ErasedSlice,
    slice_ptr::ErasedSlicePtr,
    value::{Erased, try_copy_from_slice},
};

#[cfg(feature = "alloc")]
pub use self::value::BoxedErased;

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
