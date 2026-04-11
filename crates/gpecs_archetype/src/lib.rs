//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

pub mod bundle;
pub mod erased;
pub mod registry;

#[cfg(feature = "alloc")]
mod alloc;
