//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![deny(unsafe_code)] // allow it only for key-value pair & for mutable access to keys and items
#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc as core_alloc;

pub use gpecs_soa as soa;

#[cfg(feature = "alloc")]
pub use self::alloc::{arena, set};

pub mod error;
pub mod item;
pub mod iter;
pub mod key;
pub mod pair;
pub mod prelude;
pub mod view;

#[cfg(feature = "alloc")]
mod alloc;

mod algo;
mod assert;
