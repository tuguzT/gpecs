use bytemuck::{Pod, Zeroable};
use gpecs_component::{Component, GpuComponent};

use crate::utils::RandomXoshiro128;

pub const DEFAULT_SEED: u32 = 340_383;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Data {
    pub thingy: i32,
    pub dingy: f32,
    pub mingy: u32,
    pub seed: u32,
    pub rng: RandomXoshiro128,
    pub numgy: u32,
    pub padding: [u32; 3],
}

impl Component for Data {}
impl GpuComponent for Data {}
