#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![no_std]

use spirv_std::{glam::USizeVec3, spirv};

use gpecs_soa_erased::erased::{ErasedSoa, ErasedSoaContext, ErasedSoaSlicesMut};
use gpecs_sparse::soa::{field::FieldDescriptor, slice::SoaSlicesMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ErasedSoaDesc {
    pub len: usize,
    pub capacity: usize,
}

#[spirv(compute(threads(64)))]
pub fn erased_soa_work(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] erased_soa_desc: &ErasedSoaDesc,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] dense: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] descriptors: &[FieldDescriptor],
) {
    let ErasedSoaDesc { len, capacity } = *erased_soa_desc;
    let invocation_id = id.x;

    let context = unsafe { ErasedSoaContext::new_unchecked(descriptors) };
    let slices = unsafe {
        ErasedSoaSlicesMut::new_unchecked(descriptors, dense.as_mut_ptr(), capacity, 0, len)
    };
    let slices = SoaSlicesMut::<ErasedSoa<&[u32], _, u32>>::new(&context, slices);

    let ptrs = unsafe { slices.get_unchecked(invocation_id) };
    dense[invocation_id] = u32::try_from(ptrs.offset()).unwrap_or_default();
}
