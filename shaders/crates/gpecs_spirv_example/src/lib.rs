#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(unexpected_cfgs)] // to not warn about `spirv` target arch
#![deny(warnings)] // to see warnings from `spirv-builder` builds
#![forbid(unsafe_op_in_unsafe_fn)]
#![feature(asm_experimental_arch)]

use spirv_std::{arch::IndexUnchecked, glam::UVec3, spirv};

// (x = 64, y = 1, z = 1)
#[spirv(compute(threads(64)))]
pub fn compute_shader(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] data: &mut [u32],
) {
    let mut index = 0;
    while index < 5 {
        let item = unsafe { data.index_unchecked_mut(index) };
        *item = id.x;

        index += 1;
    }
}
