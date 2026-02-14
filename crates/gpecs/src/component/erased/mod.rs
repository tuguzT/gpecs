pub use self::{
    mut_ptr::ErasedComponentMutPtr, mut_ref::ErasedComponentRefMut,
    mut_slice::ErasedComponentSliceMut, mut_slice_ptr::ErasedComponentSliceMutPtr,
    nonnull_ptr::ErasedComponentNonNullPtr, ptr::ErasedComponentPtr, r#ref::ErasedComponentRef,
    slice::ErasedComponentSlice, slice_ptr::ErasedComponentSlicePtr, value::ErasedComponent,
};

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
