//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use gpecs_soa as soa;

pub mod aligned_bytes;
pub mod erased;
pub mod error;
pub mod field;

mod fmt;
