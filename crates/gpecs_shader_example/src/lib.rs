#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]

use gpecs_soa::prelude::*;
use gpecs_types::entity::Entity;
use spirv_std::{glam::UVec3, spirv};
use static_assertions as sa;
use unroll::unroll_for_loops;

const WORKGROUP_SIZE: usize = 256;

#[allow(unused)] // unroll does not support const variables, so integer literal is used instead
const ITER_COUNT: usize = WORKGROUP_SIZE / 32;
sa::const_assert_eq!(ITER_COUNT, 8);

#[spirv(compute(threads(64)))]
#[unroll_for_loops]
pub fn copy_entity_indices(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] entities: &[Entity],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] indices: &mut [u32],
) {
    let (entities,): <(_,) as Soa>::Slices<'_> = (entities,);
    let index = id.x as usize;
    indices[index] = entities[index].index();
}
