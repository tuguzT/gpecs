use gpecs_types::component::{Component, GpuComponent};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C, align(8))]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Default for Velocity {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

impl Component for Velocity {}
impl GpuComponent for Velocity {}
