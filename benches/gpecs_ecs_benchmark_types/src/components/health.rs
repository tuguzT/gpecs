use bytemuck::{NoUninit, Zeroable};
use gpecs_types::component::{Component, GpuComponent};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroable, NoUninit)]
#[repr(u32)]
pub enum StatusEffect {
    #[default]
    Spawn,
    Dead,
    Alive,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroable, NoUninit)]
#[repr(C, align(16))]
pub struct Health {
    pub hp: i32,
    pub max_hp: i32,
    pub status: StatusEffect,
    pub padding: u32,
}

impl Component for Health {}
impl GpuComponent for Health {}
