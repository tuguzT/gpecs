//! Nothing too special, too =)

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![deny(unsafe_code)] // allow it only for some method definitions
#![forbid(unsafe_op_in_unsafe_fn)]
#![no_std]

pub use gpecs_sparse::soa;

pub mod archetype;
pub mod component;
pub mod entity;
pub mod world;
