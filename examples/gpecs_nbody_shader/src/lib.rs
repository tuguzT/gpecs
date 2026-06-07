#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![cfg_attr(not(test), no_std)]

use gpecs_nbody_types::CameraBuffer;
use spirv_std::{
    glam::{Vec3, Vec4},
    spirv,
};

#[spirv(vertex)]
pub fn vertex(
    in_position: Vec3,
    in_color: Vec3,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] camera: &CameraBuffer,
) {
    *out_position = camera.model_view_projection * in_position.extend(1.0);
    *out_color = in_color;
}

#[spirv(fragment)]
pub fn fragment(in_color: Vec3, out_color: &mut Vec4) {
    *out_color = in_color.extend(1.0);
}

#[spirv(compute(threads(64)))]
pub fn system() {}
