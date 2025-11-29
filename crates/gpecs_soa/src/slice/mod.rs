pub use self::{
    dst::{SoaSlice, from_raw_parts, from_raw_parts_mut},
    index::{SoaSlicePtrsIndex, SoaSlicesIndex, range},
    iter::{Iter, IterMut},
    slices::{SoaSlices, SoaSlicesMut},
};

pub(crate) use self::{
    index::{IndexHelper, IndexHelperMut},
    partial_eq::partial_eq_impl,
    partial_ord::partial_ord_impl,
};

mod assert;
mod dst;
mod index;
mod iter;
mod partial_eq;
mod partial_ord;
mod slices;
