//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_op_in_unsafe_fn)]
#![no_std]

pub mod components;
pub mod framebuffer;
pub mod systems;
pub mod utils;
