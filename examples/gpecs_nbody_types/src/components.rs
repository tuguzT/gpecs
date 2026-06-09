use bytemuck::{NoUninit, Pod, Zeroable};
use glam::Vec3;
use gpecs_component::{Component, GpuComponent};

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Position {
    pub data: Vec3,
    pub padding: u32,
}

impl Component for Position {}
impl GpuComponent for Position {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Velocity {
    pub data: Vec3,
    pub padding: u32,
}

impl Component for Velocity {}
impl GpuComponent for Velocity {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Force {
    pub data: Vec3,
    pub padding: u32,
}

impl Component for Force {}
impl GpuComponent for Force {}

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, Zeroable)]
#[repr(transparent)]
pub struct Mass(f32);

impl Mass {
    pub fn new(value: f32) -> Option<Self> {
        if value <= 0.0 {
            return None;
        }
        Some(Self(value))
    }

    pub fn as_f32(self) -> f32 {
        let Self(value) = self;
        value
    }
}

impl Component for Mass {}
impl GpuComponent for Mass {}

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, Zeroable)]
#[repr(transparent)]
pub struct Radius(f32);

impl Radius {
    pub fn new(value: f32) -> Option<Self> {
        if value <= 0.0 {
            return None;
        }
        Some(Self(value))
    }

    pub fn as_f32(self) -> f32 {
        let Self(value) = self;
        value
    }
}

impl Component for Radius {}
impl GpuComponent for Radius {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Color {
    pub rgb_unorm: Vec3,
    pub padding: u32,
}

impl Component for Color {}
impl GpuComponent for Color {}
