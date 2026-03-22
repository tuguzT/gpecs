use bytemuck::{Pod, Zeroable};
use gpecs_component::{Component, GpuComponent};

pub const PLAYER_SPRITE: u32 = '@' as u32;
pub const MONSTER_SPRITE: u32 = 'k' as u32;
pub const NPC_SPRITE: u32 = 'h' as u32;
pub const GRAVE_SPRITE: u32 = '|' as u32;
pub const SPAWN_SPRITE: u32 = '_' as u32;
pub const NONE_SPRITE: u32 = ' ' as u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
#[repr(transparent)]
pub struct Sprite {
    pub character: u32,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            character: NONE_SPRITE,
        }
    }
}

impl Component for Sprite {}
impl GpuComponent for Sprite {}
