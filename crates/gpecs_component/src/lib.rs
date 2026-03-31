//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc as core_alloc;

pub use self::traits::{Component, GpuComponent};

pub mod erased;
pub mod registry;

mod traits;

#[cfg(feature = "alloc")]
mod alloc;
