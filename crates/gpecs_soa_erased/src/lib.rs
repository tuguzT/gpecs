//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub use gpecs_soa as soa;

pub mod align;
pub mod erased;
pub mod error;
pub mod field;

mod assert;
mod byte;
