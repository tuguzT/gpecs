//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc as core_alloc;

pub mod bundle;
pub mod erased;
pub mod registry;
