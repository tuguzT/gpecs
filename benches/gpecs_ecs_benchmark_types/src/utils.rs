use core::ops::Range;

use bytemuck::{Pod, Zeroable};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Pod, Zeroable)]
#[repr(transparent)]
pub struct TimeDelta(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
#[repr(transparent)]
pub struct RandomXoshiro128 {
    state: [u32; 4],
}

impl RandomXoshiro128 {
    pub fn new(seed: u32) -> Self {
        let state = [seed + 3, seed + 5, seed + 7, seed + 11];
        Self { state }
    }

    pub fn from_state(state: [u32; 4]) -> Self {
        Self { state }
    }

    pub fn generate(&mut self) -> u32 {
        let Self { state } = self;

        let result = u32::rotate_left(state[1] * 5, 7) * 9;

        let t = state[1] << 9;
        state[2] ^= state[0];
        state[3] ^= state[1];
        state[1] ^= state[2];
        state[0] ^= state[3];
        state[2] ^= t;
        state[3] = u32::rotate_left(state[3], 11);

        result
    }

    pub fn range(&mut self, range: Range<u32>) -> u32 {
        let Range {
            start: low,
            end: high,
        } = range;

        let r = high - low + 1;
        (self.generate() % r) + low
    }
}
