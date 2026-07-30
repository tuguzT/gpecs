//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use gpecs_identity as identity;

pub mod mem;
pub mod ptr;
pub mod slice;
pub mod traits;
pub mod wrapper;
