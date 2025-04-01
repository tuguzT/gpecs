#[cfg(feature = "alloc")]
pub use crate::vec::SoaVec;
pub use crate::{
    slice::SoaSlice,
    slice::{SoaSlices, SoaSlicesMut},
    traits::Soa,
};
