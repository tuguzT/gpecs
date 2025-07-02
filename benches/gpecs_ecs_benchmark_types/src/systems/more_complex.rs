use crate::components::{Data, Position, Velocity};

pub fn update_components(position: &Position, velocity: &mut Velocity, data: &mut Data) {
    if (data.thingy % 10) == 0 {
        if position.x > position.y {
            velocity.x = (data.rng.range(3..19)) as f32 - 10.0;
            velocity.y = (data.rng.range(0..5)) as f32;
        } else {
            velocity.x = (data.rng.range(0..5)) as f32;
            velocity.y = (data.rng.range(3..19)) as f32 - 10.0;
        }
    }
}
