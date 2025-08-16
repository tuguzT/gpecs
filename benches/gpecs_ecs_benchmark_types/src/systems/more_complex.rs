use num_traits::ToPrimitive;

use crate::components::{Data, Position, Velocity};

pub fn update_components(position: &Position, velocity: &mut Velocity, data: &mut Data) {
    if (data.thingy % 10) == 0 {
        if position.x > position.y {
            velocity.x = (data.rng.range(3..19)).to_f32().unwrap() - 10.0;
            velocity.y = (data.rng.range(0..5)).to_f32().unwrap();
        } else {
            velocity.x = (data.rng.range(0..5)).to_f32().unwrap();
            velocity.y = (data.rng.range(3..19)).to_f32().unwrap() - 10.0;
        }
    }
}
