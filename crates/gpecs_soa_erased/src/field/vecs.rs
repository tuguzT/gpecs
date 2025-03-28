use alloc::vec::Vec;

use crate::{align::Aligned, byte::ErasedByte, soa::traits::FieldDescriptor};

// TODO: implement some API (in the future)
pub struct ErasedFieldVec<Fields> {
    pub(crate) desc: FieldDescriptor,
    // data is stored inline in a single buffer
    pub(crate) buffer: Vec<ErasedByte<Aligned<Fields>>>,
}
