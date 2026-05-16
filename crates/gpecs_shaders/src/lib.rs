#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![cfg_attr(not(test), no_std)]
#![allow(unused_imports)]

use core::convert::Infallible;
use spirv_std::{
    TypedBuffer,
    glam::{USizeVec3, UVec3},
    spirv,
};

use bytemuck::{Pod, Zeroable};
use gpecs_entity::{Entity, EntityEpoch, EntitySparseItem};
use gpecs_soa_erased::{
    ErasedSoa, ErasedSoaContext, ErasedSoaMutSlices,
    soa::{identity::Identity, slice::SoaSlicesMut},
};
use gpecs_sparse::{
    item::{KeyValueMutSlices, KeyValuePair},
    view::EpochSparseViewMut,
};
use gpecs_world::id::WorldId;

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
        ErasedSoaMutSlices::new_unchecked(layouts, lhs_dense, capacity, 0, len)
    };
    let mut lhs_dense = SoaSlicesMut::<GpuErasedSoa<_>>::new(&context, lhs_dense);

    let rhs_dense = unsafe {
        let len = u32_to_usize(rhs_len);
        let capacity = u32_to_usize(rhs_capacity);
        ErasedSoaMutSlices::new_unchecked(layouts, rhs_dense, capacity, 0, len)
    };
    let mut rhs_dense = SoaSlicesMut::<GpuErasedSoa<_>>::new(&context, rhs_dense);

    assert!(invocation_id < lhs_dense.len());
    let mut lhs = unsafe { lhs_dense.get_unchecked_mut(invocation_id) };

    assert!(invocation_id < rhs_dense.len());
    let mut rhs = unsafe { rhs_dense.get_unchecked_mut(invocation_id) };

    unsafe { lhs.swap(&mut rhs) }
}

pub type SparseGetDescUniform = TypedBuffer<SparseGetDesc>;
pub type EntityStorage = TypedBuffer<[Entity]>;
pub type SparseStorage = TypedBuffer<[EntitySparseItem]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
#[repr(C)]
pub struct SparseGetDesc {
    len: u32,
}

#[cfg(test)]
#[spirv(compute(threads(64)))]
pub fn sparse_get(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] desc: &SparseGetDescUniform,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] layouts: &LayoutsStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] entities: &mut EntityStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] dense: &mut DenseStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] sparse: &mut SparseStorage,
) {
    let invocation_id = id.x;
    let SparseGetDesc { len } = **desc;

    let layouts = GpuFieldLayouts::from(&**layouts);
    let context = unsafe { ErasedSoaContext::from_inner(layouts) };
    let context = Identity::from_inner(context);

    let values = unsafe {
        let len = u32_to_usize(len);
        ErasedSoaMutSlices::new_unchecked(layouts, dense, len, 0, len)
    };
    let dense = unsafe { KeyValueMutSlices::<_, GpuErasedSoa<_>>::new_unchecked(entities, values) };
    let dense = SoaSlicesMut::<KeyValuePair<_, _>>::new(&context, dense);

    let mut view = unsafe { EpochSparseViewMut::from_parts(dense, sparse) };
    let entity = Entity::new(invocation_id, EntityEpoch::default(), WorldId::default());

    let _ = unsafe { view.get_unchecked_mut(entity) };
}
