use core::time::Duration;

use bytemuck::{Pod, Zeroable};
use glam::vec3;

use crate::{
    components::{Color, Force, Mass, Position, Radius, Velocity},
    render::Vertex,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Pod, Zeroable)]
#[repr(transparent)]
pub struct TimeDelta(f32);

impl TimeDelta {
    pub fn new(duration: Duration) -> Self {
        let seconds = duration.as_secs_f32();
        Self(seconds)
    }

    pub fn as_f32(self) -> f32 {
        let Self(value) = self;
        value
    }
}

pub fn nbody_force_from(position: Position, other_position: Position, other_mass: Mass) -> Force {
    let diff = other_position.data - position.data;

    let inv_dist = (diff.length() + 10.0).recip();
    let inv_dist3 = inv_dist * inv_dist * inv_dist;

    let data = diff * (other_mass.as_f32() * inv_dist3);
    Force { data, padding: 0 }
}

pub fn accelerate(force: Force, mass: Mass, velocity: &mut Velocity, delta_time: TimeDelta) {
    velocity.data += force.data / mass.as_f32() * delta_time.as_f32();
}

pub fn r#move(velocity: Velocity, position: &mut Position, delta_time: TimeDelta) {
    position.data += velocity.data * delta_time.as_f32();
}

pub fn color_from(velocity: Velocity) -> Color {
    let speed = velocity.data.length().min(1.0);
    let r = (speed - 0.2).max(0.0) / 0.8;
    let g = (speed - 0.7).max(0.0) / 0.3;
    let b = (speed * 155.0 + 100.0) / 255.0;
    Color {
        rgb_unorm: vec3(r, g, b),
        padding: 0,
    }
}

pub fn vertex_from(position: Position, color: Color, radius: Radius) -> Vertex {
    Vertex {
        position: position.data,
        size: radius.as_f32(),
        color: color.rgb_unorm,
        padding: 0,
    }
}
