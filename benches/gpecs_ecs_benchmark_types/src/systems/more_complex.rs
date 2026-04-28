use num_traits::ToPrimitive;

use crate::components::{Data, Position, Velocity};

pub fn update_components(position: &Position, velocity: &mut Velocity, data: &mut Data) {
    if (data.thingy % 10) == 0 {
        if position.data.x > position.data.y {
            velocity.data.x = (data.rng.range(3..19)).to_f32().unwrap() - 10.0;
            velocity.data.y = (data.rng.range(0..5)).to_f32().unwrap();
        } else {
            velocity.data.x = (data.rng.range(0..5)).to_f32().unwrap();
            velocity.data.y = (data.rng.range(3..19)).to_f32().unwrap() - 10.0;
        }
    }
}
