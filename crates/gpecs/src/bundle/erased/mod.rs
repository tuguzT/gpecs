pub use gpecs_archetype::bundle::erased::*;

pub use self::value::{
    ErasedBorrowedBundle, ErasedBorrowedBundleIntoIter, ErasedBorrowedViewBundle,
    ErasedBorrowedViewBundleIntoIter, ErasedBundle, ErasedBundleIntoIter, ErasedBundleIntoIterKind,
    ErasedBundleKind, FromErasedComponent, RemovePair, ShuffledBundle,
};

pub mod error;

mod soa_impl;
mod value;
