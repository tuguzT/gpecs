//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(feature = "alloc")]
extern crate alloc as core_alloc;

pub use gpecs_layout as layout;
pub use gpecs_soa_core::{identity, mem, wrapper};

#[cfg(feature = "alloc")]
pub use self::alloc::vec;

pub mod field;
pub mod prelude;
pub mod ptr;
pub mod slice;
pub mod traits;

mod buffer;

#[cfg(feature = "alloc")]
mod alloc;
