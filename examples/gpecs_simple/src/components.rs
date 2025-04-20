use glam::Vec3;
use gpecs::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(align(16))]
pub struct Position {
    pub data: Vec3,
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
