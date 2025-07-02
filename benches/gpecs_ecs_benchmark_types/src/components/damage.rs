use gpecs_types::component::{Component, GpuComponent};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct Damage {
    pub attack: i32,
    pub defense: i32,
}

impl Component for Damage {}
impl GpuComponent for Damage {}
