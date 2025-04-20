use crate::components::{Health, Player, PlayerType, Sprite, StatusEffect};

pub const PLAYER_SPRITE: u32 = '@' as u32;
pub const MONSTER_SPRITE: u32 = 'k' as u32;
pub const NPC_SPRITE: u32 = 'h' as u32;
pub const GRAVE_SPRITE: u32 = '|' as u32;
pub const SPAWN_SPRITE: u32 = '_' as u32;
pub const NONE_SPRITE: u32 = ' ' as u32;

pub fn update_sprite(sprite: &mut Sprite, player: &Player, health: &Health) {
    sprite.character = match health.status {
        StatusEffect::Alive => match player.r#type {
            PlayerType::Hero => PLAYER_SPRITE,
            PlayerType::Monster => MONSTER_SPRITE,
            PlayerType::NPC => NPC_SPRITE,
        },
        StatusEffect::Dead => GRAVE_SPRITE,
        StatusEffect::Spawn => SPAWN_SPRITE,
    };
}
