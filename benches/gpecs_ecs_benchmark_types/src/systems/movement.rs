use crate::{
    components::{Position, Velocity},
    utils::TimeDelta,
};

pub fn update_position(position: &mut Position, velocity: &Velocity, dt: TimeDelta) {
    position.x += velocity.x * dt.0;
    position.y += velocity.y * dt.0;
}
