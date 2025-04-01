//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use self::traits::Soa;

#[cfg(feature = "alloc")]
pub mod vec;

pub mod identity;
pub mod mem;
pub mod prelude;
pub mod ptr;
pub mod slice;
pub mod traits;

#[cfg(feature = "alloc")]
mod raw_vec;
#[cfg(feature = "alloc")]
mod set_len_on_drop;
