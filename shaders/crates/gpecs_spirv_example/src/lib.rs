#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(unexpected_cfgs)] // to not to warn about `spirv` target arch
#![deny(warnings)] // to see warnings from `spirv-builder` builds
#![feature(asm_experimental_arch)]

use spirv_std::{arch::IndexUnchecked, glam::UVec3, macros::debug_printfln, spirv};

#[allow(dead_code)]
struct SliceTest<'a, T, U, V> {
    t_slice: &'a mut [T],
    u_slice: &'a [U],
    v_slice: &'a [V],
}

// (x = 64, y = 1, z = 1)
#[spirv(compute(threads(64)))]
pub fn compute_shader(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] data_0: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] data_1: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] data_2: &[i32],
) {
    let test = SliceTest {
        t_slice: data_0,
        u_slice: data_1,
        v_slice: data_2,
    };

    let index = id.x as usize;
    unsafe {
        *test.t_slice.index_unchecked_mut(index) = id.y;
        debug_printfln!("test.u_slice[%o] = %f", id.x, test.u_slice[index]);
        debug_printfln!(
            "test.t_slice[%o] < test.t_slice[%o + 1] = %o",
            id.x,
            id.x,
            (*test.t_slice.index_unchecked_mut(index)
                < *test.t_slice.index_unchecked_mut(index + 1)) as u32,
        );
    }
}
