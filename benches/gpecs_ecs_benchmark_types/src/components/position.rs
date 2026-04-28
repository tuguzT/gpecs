use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use gpecs_component::{Component, GpuComponent};

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(8))]
pub struct Position {
    pub data: Vec2,
}

impl Component for Position {}
impl GpuComponent for Position {}
