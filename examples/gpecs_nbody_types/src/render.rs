use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Vertex {
    pub position: Vec3,
    pub size: f32,
    pub color: Vec3,
    pub padding: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct UniformBuffer {
    pub model_view_projection: Mat4,
    pub resolution: Vec4,
}
