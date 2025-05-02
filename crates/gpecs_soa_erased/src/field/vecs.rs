use crate::{aligned_bytes::AlignedBytes, soa::FieldDescriptor};

// TODO: implement some API (in the future)
pub struct ErasedFieldVec {
    pub(crate) desc: FieldDescriptor,
    // data is stored inline in a single buffer
    pub(crate) buffer: AlignedBytes,
}
