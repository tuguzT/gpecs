//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use gpecs_ptr as ptr;

pub mod data;
pub mod error;
pub mod storage;
