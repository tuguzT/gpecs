#![cfg_attr(not(test), no_std)]

use gpecs_ecs_benchmark_types::{
    components::{Damage, Data, Health, Player, Position, Sprite, Velocity},
    framebuffer::{Framebuffer, FramebufferDesc},
    systems,
    utils::TimeDelta,
};
use spirv_std::{glam::USizeVec3, spirv};

#[spirv(compute(threads(64)))]
pub fn update_damage(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] health: &mut [Health],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] damage: &[Damage],
) {
    let index = id.x;
    let health = &mut health[index];
    let damage = &damage[index];
    systems::update_damage(health, damage);
}

#[spirv(compute(threads(64)))]
pub fn update_data(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] data: &mut [Data],
    #[spirv(uniform, descriptor_set = 0, binding = 1)] dt: &TimeDelta,
) {
    let index = id.x;
    let data = &mut data[index];
    systems::update_data(data, *dt);
}

#[spirv(compute(threads(64)))]
pub fn update_health(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] health: &mut [Health],
) {
    let index = id.x;
    let health = &mut health[index];
    systems::update_health(health);
}

#[spirv(compute(threads(64)))]
pub fn update_components(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] position: &[Position],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] velocity: &mut [Velocity],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] data: &mut [Data],
) {
    let index = id.x;
    let position = &position[index];
    let velocity = &mut velocity[index];
    let data = &mut data[index];
    systems::update_components(position, velocity, data);
}

#[spirv(compute(threads(64)))]
pub fn update_position(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] position: &mut [Position],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] velocity: &[Velocity],
    #[spirv(uniform, descriptor_set = 0, binding = 2)] dt: &TimeDelta,
) {
    let index = id.x;
    let position = &mut position[index];
    let velocity = &velocity[index];
    systems::update_position(position, velocity, *dt);
}

#[spirv(compute(threads(64)))]
pub fn render_sprite(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] position: &[Position],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] sprite: &[Sprite],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] framebuffer_data: &mut [u32],
    #[spirv(uniform, descriptor_set = 0, binding = 3)] framebuffer_desc: &FramebufferDesc,
) {
    let index = id.x;
    let position = &position[index];
    let sprite = &sprite[index];

    let FramebufferDesc { width, height } = *framebuffer_desc;
    let mut framebuffer = Framebuffer::new(width, height, framebuffer_data);
    systems::render_sprite(position, sprite, &mut framebuffer);
}

#[spirv(compute(threads(64)))]
pub fn update_sprite(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] sprite: &mut [Sprite],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] player: &[Player],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] health: &[Health],
) {
    let index = id.x;
    let sprite = &mut sprite[index];
    let player = &player[index];
    let health = &health[index];
    systems::update_sprite(sprite, player, health);
}
