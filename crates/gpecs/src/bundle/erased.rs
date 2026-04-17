use std::mem::MaybeUninit;

use gpecs_archetype::bundle::erased::{self, traits::MustDrop};
use gpecs_soa_erased::{ptr::slice::CoreSliceItemPtrs, storage::BoxedAlignedUninitStorage};

pub use erased::*;

pub type ErasedBundleRefs<'a, D> = erased::ErasedBundleRefs<'a, D, *const MaybeUninit<u8>>;
pub type ErasedBundleMutRefs<'a, D> = erased::ErasedBundleMutRefs<'a, D, *mut MaybeUninit<u8>>;

pub type ErasedBundleKind<T> = erased::ErasedBundleKind<T, MustDrop, Storage, SlicePtrs>;
pub type ErasedBundle<Meta> = erased::ErasedBundle<Meta, MustDrop, Storage, SlicePtrs>;
pub type ErasedBorrowedBundle<'a, Meta> =
    erased::ErasedBorrowedBundle<'a, Meta, MustDrop, Storage, SlicePtrs>;
pub type ErasedBorrowedViewBundle<'a, Meta> =
    erased::ErasedBorrowedViewBundle<'a, Meta, MustDrop, Storage, SlicePtrs>;

type Storage = BoxedAlignedUninitStorage;
type SlicePtrs = CoreSliceItemPtrs<MaybeUninit<u8>>;
