pub use gpecs_soa_core::slices::*;

pub use self::dst::{SoaSlice, from_raw_parts, from_raw_parts_mut};

#[cfg(feature = "alloc")]
pub use crate::alloc::slice::ToSoaVec;

mod dst;
mod partial_eq;
mod partial_ord;
