//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(feature = "alloc")]
extern crate alloc as core_alloc;

pub use gpecs_identity as identity;
pub use gpecs_layout as layout;

#[cfg(feature = "alloc")]
pub use self::alloc::vec;

pub mod field;
pub mod mem;
pub mod prelude;
pub mod ptr;
pub mod slice;
pub mod traits;
pub mod wrapper;

mod buffer;

#[cfg(feature = "alloc")]
mod alloc;
