use std::mem::MaybeUninit;

use gpecs_soa_erased::{ptr::slice::CoreSliceItemPtrs, storage::BoxedAlignedUninitStorage};

pub use gpecs_component::erased::*;

pub type ErasedComponent = gpecs_component::erased::ErasedComponent<
    BoxedAlignedUninitStorage,
    CoreSliceItemPtrs<MaybeUninit<u8>>,
>;
