pub use self::{
    index::{SlicePtrsIndex, range, try_range},
    iter::IterPtrs,
    iter_mut::IterMutPtrs,
    raw::*,
    view::SoaSlicePtrs,
    view_mut::SoaSliceMutPtrs,
};

mod index;
mod iter;
mod iter_mut;
mod raw;
mod view;
mod view_mut;
