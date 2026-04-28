use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use gpecs_component::{Component, GpuComponent};

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(8))]
pub struct Velocity {
    pub data: Vec2,
}

impl Default for Velocity {
    fn default() -> Self {
        let data = Vec2::splat(1.0);
        Self { data }
    }
}

impl Component for Velocity {}
impl GpuComponent for Velocity {}
