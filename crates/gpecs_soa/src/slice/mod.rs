pub use self::{
    dst::{SoaSlice, from_raw_parts, from_raw_parts_mut},
    index::{SoaSliceIndex, range},
    iter::{Iter, IterMut},
    slices::{SoaSlices, SoaSlicesMut},
};

pub(crate) use self::index::{IndexHelper, IndexHelperMut};

mod assert;
mod dst;
mod index;
mod iter;
mod slices;
