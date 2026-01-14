#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![no_std]

use gpecs_types::soa::field::FieldDescriptor;
use spirv_std::{glam::UVec3, spirv};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErasedSoaBufferDesc {
    pub len: usize,
    pub capacity: usize,
}

#[spirv(compute(threads(64)))]
pub fn erased_soa_work(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] buffer_desc: &ErasedSoaBufferDesc,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] field_descs: &[FieldDescriptor],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] buffer: &mut [u32],
) {
    let ErasedSoaBufferDesc { len, capacity } = *buffer_desc;
    let index = id.x as usize;

    let _ = (len, capacity, field_descs);
    buffer[index] += 1;

    // TODO: does not compile because of pointer casts
    // let _ = gpecs_soa_erased::erased::ErasedSoaSlicesMut::new(
    //     field_descs,
    //     unsafe { buffer.align_to_mut().1 },
    //     capacity,
    //     0,
    //     len,
    // );
}
