pub use self::{
    damage::update_damage, data::update_data, health::update_health,
    more_complex::update_components, movement::update_position, render::render_sprite,
    sprite::update_sprite,
};

mod damage;
mod data;
mod health;
mod more_complex;
mod movement;
mod render;
mod sprite;
