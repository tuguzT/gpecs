use std::{mem::MaybeUninit, ptr::NonNull};

use gpecs_soa_erased::{ptr::slice::CoreSliceItemPtrs, storage::BoxedAlignedUninitStorage};

pub use gpecs_component::erased::{ErasedDrop, WithErasedDrop, error};

pub type ErasedComponent = gpecs_component::erased::ErasedComponent<
    BoxedAlignedUninitStorage,
    CoreSliceItemPtrs<MaybeUninit<u8>>,
>;

pub type ErasedComponentPtr = gpecs_component::erased::ErasedComponentPtr<*const MaybeUninit<u8>>;
pub type ErasedComponentMutPtr =
    gpecs_component::erased::ErasedComponentMutPtr<*mut MaybeUninit<u8>>;
pub type ErasedComponentNonNullPtr =
    gpecs_component::erased::ErasedComponentNonNullPtr<NonNull<MaybeUninit<u8>>>;

pub type ErasedComponentSlicePtr =
    gpecs_component::erased::ErasedComponentSlicePtr<*const MaybeUninit<u8>>;
pub type ErasedComponentMutSlicePtr =
    gpecs_component::erased::ErasedComponentMutSlicePtr<*mut MaybeUninit<u8>>;

pub type ErasedComponentRef<'a> =
    gpecs_component::erased::ErasedComponentRef<'a, *const MaybeUninit<u8>>;
pub type ErasedComponentMutRef<'a> =
    gpecs_component::erased::ErasedComponentMutRef<'a, *mut MaybeUninit<u8>>;

pub type ErasedComponentSlice<'a> =
    gpecs_component::erased::ErasedComponentSlice<'a, *const MaybeUninit<u8>>;
pub type ErasedComponentMutSlice<'a> =
    gpecs_component::erased::ErasedComponentMutSlice<'a, *mut MaybeUninit<u8>>;
