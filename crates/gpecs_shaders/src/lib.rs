#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![no_std]

use gpecs_types::soa::field::FieldDescriptor;
use spirv_std::{glam::USizeVec3, spirv};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ErasedSoaBufferDesc {
    pub len: usize,
    pub capacity: usize,
}

#[spirv(compute(threads(64)))]
#[expect(
    clippy::needless_range_loop,
    reason = "rust-gpu does not support slice iterators (yet)"
)]
pub fn erased_soa_work(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] buffer_desc: &ErasedSoaBufferDesc,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] field_descs: &[FieldDescriptor],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] buffer: &mut [u32],
) {
    let ErasedSoaBufferDesc { len, capacity } = *buffer_desc;
    let buffer_index = id.x;

    for index in 0..field_descs.len() {
        let field_layout = field_descs[index].layout();
        buffer[buffer_index] += u32::try_from(field_layout.align()).unwrap_or_default();
    }

    let _ = (len, capacity, field_descs);
    // TODO: does not compile because of pointer casts in `align_to_mut`
    // let _ = gpecs_soa_erased::erased::ErasedSoaSlicesMut::new(
    //     field_descs,
    //     unsafe { buffer.align_to_mut().1 },
    //     capacity,
    //     0,
    //     len,
    // );
}
