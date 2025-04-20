use gpecs_types::component::{Component, GpuComponent};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Empty {}

impl Component for Empty {}
impl GpuComponent for Empty {}
