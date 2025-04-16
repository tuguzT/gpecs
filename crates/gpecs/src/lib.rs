//! Nothing too special =)

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![deny(unsafe_code)] // allow it only for bundle and archetype storage impls
#![forbid(unsafe_op_in_unsafe_fn)]
// TODO `#![no_std]` with `alloc` enabled

pub use gpecs_sparse::soa;

pub mod archetype;
pub mod bundle;
pub mod component;
pub mod context;
pub mod entity;
pub mod executor;
pub mod prelude;
pub mod world;
