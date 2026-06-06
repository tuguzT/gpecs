#![cfg_attr(feature = "nightly", feature(asm_experimental_arch))]
#![cfg_attr(not(test), no_std)]

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
) {
    *out_position = in_position.extend(1.0);
    *out_color = in_color;
}

#[spirv(fragment)]
pub fn fragment(in_color: Vec3, out_color: &mut Vec4) {
    *out_color = in_color.extend(1.0);
}
