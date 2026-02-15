//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use gpecs_ptr as ptr;
pub use gpecs_soa as soa;

pub mod erased;
pub mod error;
pub mod field;
pub mod storage;

mod bytes_to_items;
mod uninit;
