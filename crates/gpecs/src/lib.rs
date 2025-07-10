//! Nothing too special =)

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
