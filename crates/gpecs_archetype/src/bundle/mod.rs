pub use self::traits::{
    Bundle, BundleMutPtrs, BundleNonNullPtrs, BundlePtrs, BundleRefs, BundleRefsMut,
    BundleSliceMutPtrs, BundleSlicePtrs, BundleSlices, BundleSlicesMut,
};

pub mod erased;
pub mod error;

mod impls;
mod traits;
