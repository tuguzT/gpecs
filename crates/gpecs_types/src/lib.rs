//! Nothing too special, too =)

#![cfg_attr(not(test), no_std)]

pub use gpecs_sparse::soa;

pub mod archetype;
pub mod component;
pub mod entity;
pub mod world;
