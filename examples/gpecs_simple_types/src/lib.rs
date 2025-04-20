#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
#![no_std]

use gpecs_types::component::{Component, GpuComponent};
use spirv_std::glam::Vec3;

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
