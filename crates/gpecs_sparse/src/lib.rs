//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![deny(unsafe_code)] // allow it only for key-value pair struct impl
#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub use gpecs_soa as soa;

pub mod arena;
pub mod error;
pub mod item;
pub mod iter;
pub mod key;
pub mod pair;
pub mod prelude;
pub mod set;
pub mod view;

mod algo;
mod assert;
mod entry;
