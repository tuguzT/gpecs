#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
}
