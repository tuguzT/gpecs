//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc as core_alloc;

#[cfg(feature = "alloc")]
pub use self::alloc::vec;

pub mod identity;
pub mod mem;
pub mod prelude;
pub mod ptr;
pub mod slice;
pub mod traits;

#[cfg(feature = "alloc")]
mod alloc;
