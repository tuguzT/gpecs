//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub use gpecs_soa as soa;

pub mod erased;
pub mod error;
pub mod field;

mod aligned_bytes;
mod assert;
