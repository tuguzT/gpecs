//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod clear;
pub mod push;
pub mod with_capacity;
pub mod work;

// TODO: convert these into normal structs when derive macro is ready
pub type Zero = ();
pub type Tiny = (u32,);
pub type Small = (f64, f64, f64);
pub type Medium = (Small, Small, Small);
pub type Big = (Small, Small, [usize; 18], String, String);
pub type Large = (
    [u32; 32],
    [u32; 32],
    [u32; 32],
    [u32; 32],
    [u32; 32],
    [u32; 32],
    [u32; 32],
    [u32; 32],
    [u32; 32],
    [u32; 32],
);
