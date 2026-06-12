#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![cfg_attr(not(test), no_std)]

use glam::{USizeVec3, Vec2, Vec3, Vec4, vec2};
use gpecs_nbody_types::{
    components::{Color, Force, Mass, Position, Radius, Velocity},
    render::{UniformBuffer, Vertex},
    systems::{TimeDelta, accelerate, color_from, r#move, nbody_force_from, vertex_from},
};
use spirv_std::{arch::kill, spirv};

#[spirv(vertex)]
#[expect(clippy::too_many_arguments, reason = "entry point")]
pub fn vertex(
    #[spirv(vertex_index)] vertex_index: usize,
    in_position: Vec3,
    in_size: f32,
    in_color: Vec3,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] uniform: &UniformBuffer,
) {
    const QUAD: [Vec2; 6] = [
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, -1.0),
        vec2(1.0, 1.0),
    ];

    let uv = QUAD[vertex_index];
    let clip_position = uniform.model_view_projection * in_position.extend(1.0);
    let point_position = (uv * in_size).extend(0.0).extend(0.0) / uniform.resolution;

    *out_position = clip_position + point_position;
    *out_color = in_color;
    *out_uv = uv;
}

#[spirv(fragment)]
pub fn fragment(in_color: Vec3, in_uv: Vec2, out_color: &mut Vec4) {
    if in_uv.length() > 1.0 {
        kill();
    }
    *out_color = in_color.extend(1.0);
}

#[spirv(compute(threads(256)))]
pub fn update_force(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] positions: &[Position],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] masses: &[Mass],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] forces: &mut [Force],
) {
    let index = id.x;
    let position = positions[index];

    let mut force = Force::default();
    for other_index in 0..positions.len() {
        if index == other_index {
            continue;
        }

        let other_position = positions[other_index];
        let other_mass = masses[other_index];
        force.data += nbody_force_from(position, other_position, other_mass).data;
    }

    forces[index] = force;
}

#[spirv(compute(threads(64)))]
pub fn update_velocity_and_position(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] forces: &[Force],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] masses: &[Mass],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] velocities: &mut [Velocity],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] positions: &mut [Position],
    #[spirv(uniform, descriptor_set = 0, binding = 4)] delta_time: &TimeDelta,
) {
    let index = id.x;
    let force = forces[index];
    let mass = masses[index];
    let velocity = &mut velocities[index];
    let position = &mut positions[index];

    accelerate(force, mass, velocity, *delta_time);
    r#move(*velocity, position, *delta_time);
}

#[spirv(compute(threads(64)))]
pub fn update_color(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] velocities: &[Velocity],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] colors: &mut [Color],
) {
    let index = id.x;
    let velocity = velocities[index];

    colors[index] = color_from(velocity);
}

#[spirv(compute(threads(64)))]
pub fn update_vertex(
    #[spirv(global_invocation_id)] id: USizeVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] positions: &[Position],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] colors: &[Color],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] radiuses: &[Radius],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] vertices: &mut [Vertex],
) {
    let index = id.x;
    let position = positions[index];
    let color = colors[index];
    let radius = radiuses[index];

    vertices[index] = vertex_from(position, color, radius);
}
