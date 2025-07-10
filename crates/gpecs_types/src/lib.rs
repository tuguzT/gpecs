//! Nothing too special, too =)

#![deny(unsafe_code)]
#![no_std]

pub use gpecs_sparse::soa;

pub mod archetype;
pub mod component;
pub mod entity;
pub mod world;
