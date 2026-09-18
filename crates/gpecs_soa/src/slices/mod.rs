pub use gpecs_soa_core::slices::*;

pub use self::dst::SoaSlice;

#[cfg(feature = "alloc")]
pub use crate::alloc::slices::ToSoaVec;

mod dst;
mod partial_eq;
mod partial_ord;
