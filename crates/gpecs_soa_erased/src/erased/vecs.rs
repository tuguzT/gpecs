use alloc::boxed::Box;

use crate::field::ErasedFieldVec;

// TODO: implement some API (in the future)
pub struct ErasedSoaVecs<Fields> {
    pub(crate) len: usize,
    pub(crate) vecs: Box<[ErasedFieldVec<Fields>]>,
}
