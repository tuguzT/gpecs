#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![feature(asm_experimental_arch)]

use gpecs_soa::prelude::*;
use gpecs_types::entity::Entity;
use spirv_std::{arch::IndexUnchecked, glam::UVec3, spirv, TypedBuffer};
use static_assertions as sa;
use unroll::unroll_for_loops;

const WORKGROUP_SIZE: usize = 256;

#[allow(unused)] // unroll does not support const variables, so integer literal is used instead
const ITER_COUNT: usize = WORKGROUP_SIZE / 32;
sa::const_assert_eq!(ITER_COUNT, 8);

#[spirv(compute(threads(64)))]
#[unroll_for_loops]
pub fn compute_shader(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] data: &mut TypedBuffer<[Entity]>,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] data_zst: &mut TypedBuffer<[()]>,
) {
    let data: <(_,) as Soa>::SlicesMut<'_> = (data,);
    for index in 0..8 {
        let item = unsafe { data.0.index_unchecked_mut(index) };
        item.set_index(index as u32 + id.x);

        let zst = unsafe { data_zst.index_unchecked_mut(index) };
        *zst = ();
    }
}
