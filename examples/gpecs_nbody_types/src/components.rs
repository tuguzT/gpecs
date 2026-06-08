use bytemuck::{Pod, Zeroable};
use glam::Vec3;

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Position {
    pub data: Vec3,
    pub padding: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Velocity {
    pub data: Vec3,
    pub padding: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Force {
    pub data: Vec3,
    pub padding: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(transparent)]
pub struct Mass(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(transparent)]
pub struct Radius(pub f32);

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Color {
    pub rgb_unorm: Vec3,
    pub padding: u32,
}
