use bytemuck::{Pod, Zeroable};
use glam::vec3;

use crate::{
    components::{Color, Force, Mass, Position, Radius, Velocity},
    render::Vertex,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Pod, Zeroable)]
#[repr(transparent)]
pub struct TimeDelta(pub f32);

pub fn nbody_force(index: usize, positions: &[Position], masses: &[Mass]) -> Force {
    let position = positions[index];

    let mut force = Force::default();
    for other_index in 0..positions.len() {
        if index == other_index {
            continue;
        }
        let other_position = positions[other_index];
        let other_mass = masses[other_index];

        let diff = other_position.data - position.data;
        force.data += diff * (other_mass.0 / diff.length());
    }
    force
}

pub fn accelerate(force: Force, mass: Mass, velocity: &mut Velocity, delta_time: TimeDelta) {
    velocity.data += force.data / mass.0 * delta_time.0;
}

pub fn r#move(velocity: Velocity, position: &mut Position, delta_time: TimeDelta) {
    position.data -= velocity.data * delta_time.0;
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
        size: radius.0,
        color: color.rgb_unorm,
        padding: 0,
    }
}
