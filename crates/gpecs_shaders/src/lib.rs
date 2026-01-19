#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![no_std]

use spirv_std::{glam::USizeVec3, spirv};

use gpecs_soa_erased::erased::ErasedSoaSlicesMut;
use gpecs_types::soa::field::FieldDescriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ErasedSoaBufferDesc {
    pub len: usize,
    pub capacity: usize,
}

#[spirv(compute(threads(64)))]
pub fn erased_soa_work(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] buffer_desc: &ErasedSoaBufferDesc,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] field_descs: &[FieldDescriptor],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] buffer: &mut [u32],
) {
    let ErasedSoaBufferDesc { len, capacity } = *buffer_desc;
    let buffer_index = id.x;

    let slices = unsafe {
        ErasedSoaSlicesMut::new_unchecked(field_descs, buffer.as_mut_ptr(), capacity, 0, len)
    };
    buffer[buffer_index] += u32::try_from(slices.offset()).unwrap_or_default();
}
