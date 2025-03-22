use alloc::{boxed::Box, vec::Vec};

use crate::traits::FieldDescriptor;

use super::byte::{Aligned, ErasedByte};

// TODO: implement some API (in the future)
pub struct ErasedFieldVec<Fields> {
    pub(super) desc: FieldDescriptor,
    // data is stored inline in a single buffer
    pub(super) buffer: Vec<ErasedByte<Aligned<Fields>>>,
}

pub struct ErasedSoaVecs<Fields> {
    pub(super) len: usize,
    pub(super) vecs: Box<[ErasedFieldVec<Fields>]>,
}
