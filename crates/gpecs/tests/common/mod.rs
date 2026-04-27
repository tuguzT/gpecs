use bytemuck::{Pod, Zeroable};
use gpecs::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub padding: u32,
}

impl Component for Position {}
impl GpuComponent for Position {}

#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct Mass {
    pub value: u32,
}

impl Component for Mass {}
impl GpuComponent for Mass {}

#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct Tag;

impl Component for Tag {}
impl GpuComponent for Tag {}
