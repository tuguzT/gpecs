use crate::{
    components::{Position, Velocity},
    utils::TimeDelta,
};

pub fn update_position(position: &mut Position, velocity: &Velocity, dt: TimeDelta) {
    position.data += velocity.data * dt.0;
}
