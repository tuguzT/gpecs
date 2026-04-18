#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;
use spirv_std::{TypedBuffer, glam::USizeVec3, spirv};

use gpecs_soa_erased::{
    ErasedSoa, ErasedSoaContext, ErasedSoaMutSlicePtrs,
    soa::{field::FieldDescriptor, slice::SoaSliceMutPtrs},
};

pub use self::{
    descriptors::GpuFieldDescriptors,
    ptr::{GpuSliceItemPtr, GpuSliceItemPtrs},
};

mod descriptors;
mod ptr;

pub type GpuErasedSoa<D> = ErasedSoa<Infallible, D, GpuSliceItemPtrs<u32>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GpuErasedSoaDesc {
    pub len: usize,
    pub capacity: usize,
}

pub type DenseStorage = TypedBuffer<[u32]>;
pub type DescriptorsStorage = TypedBuffer<[FieldDescriptor]>;

#[spirv(compute(threads(64)))]
pub fn erased_soa_work(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] erased_soa_desc: &GpuErasedSoaDesc,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] dense: &mut DenseStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] descriptors: &DescriptorsStorage,
) {
    let dense = &mut **dense;
    let descriptors = &**descriptors;

    let descriptors = GpuFieldDescriptors::from(descriptors);
    let GpuErasedSoaDesc { len, capacity } = *erased_soa_desc;
    let invocation_id = id.x;

    let context = unsafe { ErasedSoaContext::from_inner(descriptors) };
    let slices =
        unsafe { ErasedSoaMutSlicePtrs::new_unchecked(descriptors, dense, capacity, 0, len) };

    let mut dense_soa = SoaSliceMutPtrs::<GpuErasedSoa<_>>::new(&context, slices);
    unsafe { dense_soa.swap_unchecked(invocation_id, invocation_id + 64) }
}
