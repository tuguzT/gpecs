pub use self::{
    damage::update_damage,
    data::update_data,
    health::update_health,
    more_complex::update_components,
    movement::update_position,
    render::render_sprite,
    sprite::{
        update_sprite, GRAVE_SPRITE, MONSTER_SPRITE, NONE_SPRITE, NPC_SPRITE, PLAYER_SPRITE,
        SPAWN_SPRITE,
    },
};

mod damage;
mod data;
mod health;
mod more_complex;
mod movement;
mod render;
mod sprite;
