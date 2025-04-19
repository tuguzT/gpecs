#[cfg(feature = "alloc")]
pub use crate::vec::SoaVec;
pub use crate::{
    identity::Identity,
    slice::SoaSlice,
    slice::{SoaSlices, SoaSlicesMut},
    traits::Soa,
};
