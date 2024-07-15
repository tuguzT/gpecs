//! Nothing too special, too =)

use self::{id::id, resource::ResourceDesc};

pub mod resource;

mod id;

#[derive(Debug, PartialEq, Eq)]
pub struct Vertex {
    pub inputs: Vec<ResourceDesc>,
    pub outputs: Vec<ResourceDesc>,
}
