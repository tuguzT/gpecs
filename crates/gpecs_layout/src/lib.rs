//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

pub use self::{slice::bytes_to_items, traits::WithLayout};

mod slice;
mod traits;
