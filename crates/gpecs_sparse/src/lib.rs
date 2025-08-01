//! Nothing too special for now...

#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc as core_alloc;

pub use gpecs_soa as soa;

#[cfg(feature = "alloc")]
pub use self::alloc::{TryInsertAccess, arena, set};

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
