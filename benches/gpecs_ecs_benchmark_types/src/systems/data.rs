use crate::{components::Data, utils::TimeDelta};

pub fn update_data(data: &mut Data, dt: TimeDelta) {
    data.thingy = (data.thingy + 1) % 1_000_000;
    data.dingy += 0.0001 * dt.0;
    data.mingy = !data.mingy;
    data.numgy = data.rng.generate();
}
