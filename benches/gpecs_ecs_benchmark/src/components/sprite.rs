use gpecs_types::component::{Component, GpuComponent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Sprite {
    pub character: u32,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            character: ' ' as u32,
        }
    }
}

impl Component for Sprite {}
impl GpuComponent for Sprite {}
