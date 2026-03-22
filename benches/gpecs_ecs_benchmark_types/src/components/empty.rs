use bytemuck::{Pod, Zeroable};
use gpecs_component::{Component, GpuComponent};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
#[repr(transparent)]
pub struct Empty {}

impl Component for Empty {}
impl GpuComponent for Empty {}
