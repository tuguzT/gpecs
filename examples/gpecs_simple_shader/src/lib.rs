#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]

use gpecs_simple_types::*;
use gpecs_types::{entity::Entity, soa::prelude::*};
use spirv_std::{
    glam::{UVec3, Vec3},
    spirv,
};
use unroll::unroll_for_loops;

#[spirv(compute(threads(64)))]
#[unroll_for_loops]
pub fn update_entity_position(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] entities: &mut [Entity],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] positions: &mut [Position],
) {
    let (entities,): <(_,) as Soa>::Slices<'_> = (entities,);

    let index = id.x as usize;
    let entity = entities[index];
    let position = &mut positions[index];

    position.data = Vec3 {
        x: entity.index() as f32,
        y: (entity.index() as f32) / 2.0,
        z: -(entity.index() as f32) / 2.0,
    };
}

#[spirv(compute(threads(64)))]
#[unroll_for_loops]
pub fn update_entity_mass(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] entities: &mut [Entity],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] masses: &mut [Mass],
) {
    let (entities,): <(_,) as Soa>::Slices<'_> = (entities,);

    let index = id.x as usize;
    let entity = entities[index];
    let mass = &mut masses[index];

    mass.value = entity.index() + (index as u32);
}
