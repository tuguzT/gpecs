//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

pub use self::{
    layout::FfiLayout,
    repeat::{repeat, repeat_packed},
    slice::bytes_to_items,
    traits::WithLayout,
};

mod layout;
mod repeat;
mod slice;
mod traits;
