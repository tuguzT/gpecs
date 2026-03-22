use bytemuck::{NoUninit, Zeroable};
use gpecs_component::{Component, GpuComponent};

use crate::utils::RandomXoshiro128;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroable, NoUninit)]
#[repr(u32)]
pub enum PlayerType {
    #[default]
    NPC,
    Monster,
    Hero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroable, NoUninit)]
#[repr(C, align(16))]
pub struct Player {
    pub rng: RandomXoshiro128,
    pub r#type: PlayerType,
    pub padding: [u32; 3],
}

impl Component for Player {}
impl GpuComponent for Player {}
