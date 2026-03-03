#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![cfg_attr(not(test), no_std)]

use core::{convert::Infallible, mem::MaybeUninit};
use spirv_std::{glam::USizeVec3, spirv};

use gpecs_soa_erased::{
    ErasedSoa, ErasedSoaContext, ErasedSoaMutSlices,
    soa::{field::FieldDescriptor, slice::SoaSlicesMut},
};

pub use self::{
    descriptors::GpuFieldDescriptors,
    ptr::{GpuSliceItemPtr, GpuSliceItemPtrs},
};

mod descriptors;
mod ptr;

pub type GpuErasedSoa<D> = ErasedSoa<Infallible, D, GpuSliceItemPtrs<MaybeUninit<u32>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GpuErasedSoaDesc {
    pub len: usize,
    pub capacity: usize,
}

#[spirv(compute(threads(64)))]
pub fn erased_soa_work(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] erased_soa_desc: &GpuErasedSoaDesc,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] dense: &mut [MaybeUninit<u32>],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] descriptors: &[FieldDescriptor],
) {
    let descriptors = GpuFieldDescriptors::from(descriptors);
    let GpuErasedSoaDesc { len, capacity } = *erased_soa_desc;
    let invocation_id = id.x;

    let context = unsafe { ErasedSoaContext::from_inner(descriptors) };
    let slices = unsafe { ErasedSoaMutSlices::new_unchecked(descriptors, dense, capacity, 0, len) };
    let mut dense_soa = SoaSlicesMut::<GpuErasedSoa<_>>::new(&context, slices);

    dense_soa.swap(invocation_id, invocation_id + 1);

    dense[invocation_id].write(42);
}
