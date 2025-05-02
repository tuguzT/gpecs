//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc as core_alloc;

#[cfg(feature = "alloc")]
pub use self::alloc::vec;
pub use self::traits::Soa;

pub mod identity;
pub mod mem;
pub mod prelude;
pub mod ptr;
pub mod slice;
pub mod traits;

#[cfg(feature = "alloc")]
mod alloc;
mod wrappers;
