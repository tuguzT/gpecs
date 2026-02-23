pub use self::traits::{
    Bundle, BundleMutPtrs, BundleNonNullPtrs, BundlePtrs, BundleRefs, BundleRefsMut,
    BundleSliceMutPtrs, BundleSlicePtrs, BundleSlices, BundleSlicesMut,
};

pub mod erased;

mod impls;
mod traits;
