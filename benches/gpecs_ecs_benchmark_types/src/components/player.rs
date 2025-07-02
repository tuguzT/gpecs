use gpecs_types::component::{Component, GpuComponent};

use crate::utils::RandomXoshiro128;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum PlayerType {
    #[default]
    NPC,
    Monster,
    Hero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(16))]
pub struct Player {
    pub rng: RandomXoshiro128,
    pub r#type: PlayerType,
}

impl Component for Player {}
impl GpuComponent for Player {}
