#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use gpecs_component::{Component, GpuComponent};

#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Position {
    pub data: Vec3,
    pub padding: f32,
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
