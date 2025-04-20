use gpecs_types::component::{Component, GpuComponent};

use crate::utils::RandomXoshiro128;

pub const DEFAULT_SEED: u32 = 340383;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct Data {
    pub thingy: i32,
    pub dingy: f32,
    pub mingy: bool,
    pub seed: u32,
    pub rng: RandomXoshiro128,
    pub numgy: u32,
}

impl Component for Data {}
impl GpuComponent for Data {}
