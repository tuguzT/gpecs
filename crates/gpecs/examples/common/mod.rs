use gpecs::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C, align(16))]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Component for Position {}
impl GpuComponent for Position {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Mass {
    pub value: u32,
}

impl Component for Mass {}
impl GpuComponent for Mass {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Tag;

impl Component for Tag {}
impl GpuComponent for Tag {}

#[derive(Debug, PartialEq, Clone)]
pub struct Name {
    pub value: String,
}

impl Component for Name {}
