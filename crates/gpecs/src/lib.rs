//! Nothing too special =)

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
// TODO `#![no_std]` with `alloc` enabled

pub use gpecs_sparse::soa;

pub mod archetype;
pub mod id;
pub mod prelude;
