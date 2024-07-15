//! Nothing too special, too =)

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
// TODO `#![no_std]` with `alloc` enabled

pub use self::{
    graph::{Edge, Graph, Vertex, VertexId},
    resource::{ResourceAccess, ResourceDesc, ResourceId},
};

mod graph;
mod id;
mod resource;
