pub use self::traits::{
    Bundle, BundleMutPtrs, BundleNonNullPtrs, BundlePtrs, BundleRefs, BundleRefsMut,
    BundleSliceMutPtrs, BundleSlicePtrs, BundleSlices, BundleSlicesMut,
};

#[cfg(feature = "alloc")]
pub use self::traits::NewBundle;

mod impls;
mod traits;
