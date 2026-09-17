pub use self::{
    index::{IndexHelper, IndexHelperMut, SlicesIndex},
    iter::Iter,
    iter_mut::IterMut,
    partial_eq::partial_eq_impl,
    partial_ord::partial_ord_impl,
    raw::*,
    view::SoaSlices,
    view_mut::SoaSlicesMut,
};

#[cfg(feature = "rayon")]
pub use self::{par_iter::ParIter, par_iter_mut::ParIterMut};

mod index;
mod iter;
mod iter_mut;
mod partial_eq;
mod partial_ord;
mod raw;
mod view;
mod view_mut;

#[cfg(feature = "rayon")]
mod par_iter;
#[cfg(feature = "rayon")]
mod par_iter_mut;
