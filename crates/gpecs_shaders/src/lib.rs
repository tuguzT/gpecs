#![cfg_attr(not(test), no_std)]
#![allow(unused_imports)]

use core::convert::Infallible;
use spirv_std::{TypedBuffer, spirv};

use bytemuck::{Pod, Zeroable};
use glam::{USizeVec3, UVec3};
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

    unsafe { lhs.swap_nonoverlapping(&mut rhs, 1) }
}

pub type SparseGetDescUniform = TypedBuffer<SparseGetDesc>;
pub type EntityStorage = TypedBuffer<[Entity]>;
pub type SparseStorage = TypedBuffer<[EntitySparseItem]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
#[repr(C)]
pub struct SparseGetDesc {
    pub lhs_len: u32,
    pub lhs_capacity: u32,
    pub rhs_len: u32,
    pub rhs_capacity: u32,
}

#[spirv(compute(threads(64)))]
#[expect(clippy::too_many_arguments, reason = "entry point")]
pub fn sparse_get(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] desc: &SparseGetDescUniform,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] layouts: &LayoutsStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] lhs_entities: &mut EntityStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] lhs_dense: &mut DenseStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] lhs_sparse: &mut SparseStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] rhs_entities: &mut EntityStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] rhs_dense: &mut DenseStorage,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)] rhs_sparse: &mut SparseStorage,
) {
    let invocation_id = id.x;
    let SparseGetDesc {
        lhs_len,
        lhs_capacity,
        rhs_len,
        rhs_capacity,
    } = **desc;

    let layouts = GpuFieldLayouts::from(&**layouts);
    let context = unsafe { ErasedSoaContext::from_inner(layouts) };
    let context = Identity::from_inner(context);

    let lhs_values = unsafe {
        let len = u32_to_usize(lhs_len);
        let capacity = u32_to_usize(lhs_capacity);
        ErasedSoaMutSlices::new_unchecked(layouts, lhs_dense, capacity, 0, len)
    };
    let lhs_dense = unsafe {
        KeyValueMutSlices::<_, GpuErasedSoa<_>, _>::new_unchecked(lhs_entities, lhs_values)
    };
    let lhs_dense =
        SoaSlicesMut::<KeyValuePair<_, _, GpuSliceItemPtrs<_>>>::new(&context, lhs_dense);
    let mut lhs_view = unsafe { EpochSparseViewMut::from_parts(lhs_dense, lhs_sparse) };

    let rhs_values = unsafe {
        let len = u32_to_usize(rhs_len);
        let capacity = u32_to_usize(rhs_capacity);
        ErasedSoaMutSlices::new_unchecked(layouts, rhs_dense, capacity, 0, len)
    };
    let rhs_dense = unsafe {
        KeyValueMutSlices::<_, GpuErasedSoa<_>, _>::new_unchecked(rhs_entities, rhs_values)
    };
    let rhs_dense =
        SoaSlicesMut::<KeyValuePair<_, _, GpuSliceItemPtrs<_>>>::new(&context, rhs_dense);
    let mut rhs_view = unsafe { EpochSparseViewMut::from_parts(rhs_dense, rhs_sparse) };

    let entity = Entity::new(invocation_id, EntityEpoch::default(), WorldId::default());
    let mut lhs = unsafe { lhs_view.get_unchecked_mut(entity) };
    let mut rhs = unsafe { rhs_view.get_unchecked_mut(entity) };

    unsafe { lhs.swap_nonoverlapping(&mut rhs, 1) }
}
