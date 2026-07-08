#![cfg_attr(not(test), no_std)]

use glam::{USizeVec3, Vec3};
use gpecs_entity::Entity;
use gpecs_simple_types::{Mass, Position};
use spirv_std::{num_traits::ToPrimitive, spirv};

#[spirv(compute(threads(64)))]
pub fn update_entity_position(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] entities: &[Entity],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] positions: &mut [Position],
) {
    let index = id.x;
    let entity = entities[index];
    let position = &mut positions[index];

    position.data = Vec3 {
        x: entity.index().to_f32().unwrap(),
        y: entity.index().to_f32().unwrap() / 2.0,
        z: -entity.index().to_f32().unwrap() / 2.0,
    };
}

#[spirv(compute(threads(64)))]
pub fn update_entity_mass(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] entities: &[Entity],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] masses: &mut [Mass],
) {
    let index = id.x;
    let entity = entities[index];
    let mass = &mut masses[index];

    mass.value = entity.index() + u32::try_from(index).ok().unwrap();
}
