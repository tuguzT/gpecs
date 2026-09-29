pub use self::{
    index::{SlicePtrsIndex, range, try_range},
    iter::IterPtrs,
    iter_mut::IterMutPtrs,
    raw::*,
    view::SoaViewPtrs,
    view_mut::SoaViewMutPtrs,
};

mod index;
mod iter;
mod iter_mut;
mod raw;
mod view;
mod view_mut;
