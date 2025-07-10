use crate::components::{
    GRAVE_SPRITE, Health, MONSTER_SPRITE, NPC_SPRITE, PLAYER_SPRITE, Player, PlayerType,
    SPAWN_SPRITE, Sprite, StatusEffect,
};

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
