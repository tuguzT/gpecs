//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

pub use self::traits::{Component, GpuComponent};

pub mod id;

mod traits;
