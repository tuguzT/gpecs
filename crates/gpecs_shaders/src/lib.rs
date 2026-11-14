#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;
use spirv_std::{TypedBuffer, spirv};

use bytemuck::{Pod, Zeroable};
use glam::USizeVec3;
use gpecs_soa_erased::{ErasedSoa, ErasedSoaContext, ErasedSoaMutSlices, soa::slices::SoaViewMut};

use self::convert::u32_to_usize;

pub use self::{
    layouts::{GpuFieldLayout, GpuFieldLayouts, GpuLayout},
    ptrs::{GpuSliceItemPtr, GpuSliceItemPtrs},
};

mod convert;
mod layouts;
mod ptrs;

pub type GpuErasedSoa<D> = ErasedSoa<Infallible, D, GpuSliceItemPtrs<u32>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
#[repr(C)]
pub struct SwapErasedDesc {
    pub lhs_len: u32,
    pub lhs_capacity: u32,
    pub rhs_len: u32,
    pub rhs_capacity: u32,
}

pub type SwapErasedDescUniform = TypedBuffer<SwapErasedDesc>;
pub type DenseStorage = TypedBuffer<[u32]>;
pub type LayoutsStorage = TypedBuffer<[GpuFieldLayout]>;

#[spirv(compute(threads(64)))]
pub fn swap_erased(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] desc: &SwapErasedDescUniform,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] layouts: &LayoutsStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] lhs_dense: &mut DenseStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] rhs_dense: &mut DenseStorage,
) {
    let invocation_id = id.x;

    let SwapErasedDesc {
        lhs_len,
        lhs_capacity,
        rhs_len,
        rhs_capacity,
    } = **desc;

    let layouts = GpuFieldLayouts::from(&**layouts);
    let context = unsafe { ErasedSoaContext::from_inner(layouts) };

    let lhs_dense = unsafe {
        let len = u32_to_usize(lhs_len);
        let capacity = u32_to_usize(lhs_capacity);
        ErasedSoaMutSlices::from_parts(layouts, lhs_dense, capacity, 0, len)
    };
    let mut lhs_dense = SoaViewMut::<GpuErasedSoa<_>>::new(&context, lhs_dense);

    let rhs_dense = unsafe {
        let len = u32_to_usize(rhs_len);
        let capacity = u32_to_usize(rhs_capacity);
        ErasedSoaMutSlices::from_parts(layouts, rhs_dense, capacity, 0, len)
    };
    let mut rhs_dense = SoaViewMut::<GpuErasedSoa<_>>::new(&context, rhs_dense);

    assert!(invocation_id < lhs_dense.len());
    let mut lhs = unsafe { lhs_dense.get_unchecked_mut(invocation_id) };

    assert!(invocation_id < rhs_dense.len());
    let mut rhs = unsafe { rhs_dense.get_unchecked_mut(invocation_id) };

    unsafe { lhs.swap_nonoverlapping(&mut rhs, 1) }
}
