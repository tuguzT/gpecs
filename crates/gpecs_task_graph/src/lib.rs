//! Nothing too special, too =)

pub use self::{
    graph::{Edge, Vertex, VertexId},
    resource::{ResourceAccess, ResourceDesc, ResourceId},
};

mod graph;
mod id;
mod resource;
