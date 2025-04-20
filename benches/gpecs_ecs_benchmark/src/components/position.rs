use gpecs_types::component::{Component, GpuComponent};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C, align(8))]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Component for Position {}
impl GpuComponent for Position {}
