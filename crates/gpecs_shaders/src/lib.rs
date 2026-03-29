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
    let dense_soa = SoaSlicesMut::<GpuErasedSoa<_>>::new(&context, slices);

    // TODO: this fails to compile by `naga` with this message:
    //       "Type [14] '&[gpecs_soa_erased::gpecs_soa::field::FieldDescriptor]' is invalid; Expected data type, found [12]"
    //       issue lies in iterator API usage: if uncommented, this generates many new types with slices inside which `rust-gpu` refuses to inline
    // dense_soa.swap(invocation_id, invocation_id + 1);

    let soa_ptrs = unsafe { dense_soa.get_unchecked(invocation_id + 1) };
    let mut iter = soa_ptrs.into_iter();

    // uncomment code below to see newly generated types which `naga` fails to deal with
    // unsafe { iter.next_unchecked() };

    let field_ptr = unsafe { iter.next_unchecked() };
    dense[invocation_id] = unsafe { *field_ptr.as_ptr() };
}
