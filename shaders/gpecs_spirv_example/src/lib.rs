#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(unexpected_cfgs)] // to not to warn about `spirv` target arch
#![deny(warnings)] // to see warnings from `spirv-builder` builds

use spirv_std::{glam::UVec3, spirv};

// (x = 64, y = 1, z = 1)
#[spirv(compute(threads(64)))]
pub fn compute_shader(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] data: &mut [u32],
) {
    let index = id.x as usize;
    data[index] = id.y;
}
