use alloc::{boxed::Box, vec::Vec};

use crate::traits::FieldDescriptor;

use super::byte::ErasedByte;

// TODO: add API (and decide what to do with drops for fields)
// data is stored inline in a single buffer
pub struct ErasedFieldVec<Fields> {
    pub(super) buffer: Vec<ErasedByte<Fields>>,
    pub(super) desc: FieldDescriptor,
}

pub struct ErasedSoaVecs<Fields> {
    pub(super) len: usize,
    pub(super) vecs: Box<[ErasedFieldVec<Fields>]>,
}
