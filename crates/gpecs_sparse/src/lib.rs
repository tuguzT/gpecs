//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod arena;
pub mod item;
pub mod iter;
pub mod key;
pub mod prelude;
pub mod set;
pub mod view;

mod algo;
mod assert;
mod entry;
