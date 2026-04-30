pub use self::{
    dst::{SoaSlice, from_raw_parts, from_raw_parts_mut},
    index::{SoaSlicePtrsIndex, SoaSlicesIndex, range},
    iter::Iter,
    iter_mut::IterMut,
    raw_iter::RawIter,
    raw_iter_mut::RawIterMut,
    slice_mut_ptrs::SoaSliceMutPtrs,
    slice_ptrs::SoaSlicePtrs,
    slices::SoaSlices,
    slices_mut::SoaSlicesMut,
};

#[cfg(feature = "rayon")]
pub use self::{par_iter::ParIter, par_iter_mut::ParIterMut};

pub(crate) use self::index::{IndexHelper, IndexHelperMut};

#[cfg(feature = "alloc")]
pub(crate) use self::{partial_eq::partial_eq_impl, partial_ord::partial_ord_impl};

mod assert;
mod dst;
mod index;
mod iter;
mod iter_mut;
mod partial_eq;
mod partial_ord;
mod raw_iter;
mod raw_iter_mut;
mod slice_mut_ptrs;
mod slice_ptrs;
mod slices;
mod slices_mut;

#[cfg(feature = "rayon")]
mod par_iter;
#[cfg(feature = "rayon")]
mod par_iter_mut;
