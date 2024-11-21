//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub use self::soa::Soa;

pub mod prelude;
pub mod ptr;
pub mod slice;
pub mod vec;

mod raw_vec;
mod soa;
