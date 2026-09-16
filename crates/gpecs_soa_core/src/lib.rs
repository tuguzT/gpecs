//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use gpecs_identity as identity;

pub mod mem;
pub mod prelude;
pub mod ptrs;
pub mod refs;
pub mod slice;
pub mod slices;
pub mod traits;
pub mod wrapper;
