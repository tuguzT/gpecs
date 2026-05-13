#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;
use spirv_std::{TypedBuffer, glam::USizeVec3, spirv};

use gpecs_soa_erased::{ErasedSoa, ErasedSoaContext, ErasedSoaMutSlices, soa::slice::SoaSlicesMut};

use self::convert::u32_to_usize;

pub use self::{
    layouts::{GpuFieldLayout, GpuFieldLayouts, GpuLayout},
    ptrs::{GpuSliceItemPtr, GpuSliceItemPtrs},
};

mod convert;
mod layouts;
mod ptrs;

pub type GpuErasedSoa<D> = ErasedSoa<Infallible, D, GpuSliceItemPtrs<u32>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ErasedSoaWorkDesc {
    pub len: u32,
    pub capacity: u32,
}

pub type DescUniform = TypedBuffer<ErasedSoaWorkDesc>;
pub type DenseStorage = TypedBuffer<[u32]>;
pub type LayoutsStorage = TypedBuffer<[GpuFieldLayout]>;

#[spirv(compute(threads(64)))]
pub fn erased_soa_work(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] desc: &DescUniform,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] dense: &mut DenseStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] layouts: &LayoutsStorage,
) {
    let invocation_id = id.x;

    let dense = &mut **dense;
    let layouts = &**layouts;
    let ErasedSoaWorkDesc { len, capacity } = **desc;
    let len = u32_to_usize(len);
    let capacity = u32_to_usize(capacity);

    let layouts = GpuFieldLayouts::from(layouts);
    let context = unsafe { ErasedSoaContext::from_inner(layouts) };
    let slices = unsafe { ErasedSoaMutSlices::new_unchecked(layouts, dense, capacity, 0, len) };

    let mut dense_soa = SoaSlicesMut::<GpuErasedSoa<_>>::new(&context, slices);
    dense_soa.swap(invocation_id, invocation_id + 64);
}
