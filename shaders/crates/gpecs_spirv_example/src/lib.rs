#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(unexpected_cfgs)] // to not warn about `spirv` target arch
#![deny(warnings)] // to see warnings from `spirv-builder` builds
#![forbid(unsafe_op_in_unsafe_fn)]
#![feature(asm_experimental_arch)]

use core::mem::swap;

use spirv_std::{arch::IndexUnchecked, glam::UVec3, macros::debug_printfln, spirv};

struct SliceTest<'a, T, U, V> {
    t_slice: &'a mut [T],
    u_slice: &'a [U],
    v_slice: &'a mut [V],
}

unsafe fn bubble_sort<T>(data: &mut [T])
where
    T: PartialOrd + Copy,
{
    let n = data.len();
    for i in 0..n {
        for j in 0..n - i - 1 {
            let a = unsafe { &mut *(data.index_unchecked_mut(j) as *mut _) };
            let b = unsafe { &mut *(data.index_unchecked_mut(j + 1) as *mut _) };
            if a > b {
                swap(a, b);
            }
        }
    }
}

// (x = 64, y = 1, z = 1)
#[spirv(compute(threads(64)))]
pub fn compute_shader(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] data_0: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] data_1: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] data_2: &mut [i32],
) {
    let test = SliceTest {
        t_slice: data_0,
        u_slice: data_1,
        v_slice: data_2,
    };
    unsafe {
        bubble_sort(test.v_slice);
    }

    let index = id.x as usize;
    unsafe {
        *test.t_slice.index_unchecked_mut(index) = id.y;
        debug_printfln!("test.u_slice[%o] = %f", id.x, test.u_slice[index]);

        let result = test.v_slice.index_unchecked(index) < test.v_slice.index_unchecked(index + 1);
        debug_printfln!(
            "test.v_slice[%o] < test.v_slice[%o + 1] = %o",
            id.x,
            id.x,
            result as u32,
        );
    }
}
