//! Nothing too special =)

// TODO `#![no_std]` with `alloc` enabled

pub use gpecs_sparse::soa;
pub use gpecs_world as world;

pub mod archetype;
pub mod bundle;
pub mod component;
pub mod context;
pub mod entity;
pub mod executor;
pub mod prelude;

mod hash;
