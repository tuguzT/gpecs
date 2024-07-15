use crate::{id::new_id_type, resource::ResourceDesc};

new_id_type! {
    pub VertexId;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Vertex {
    pub inputs: Vec<ResourceDesc>,
    pub outputs: Vec<ResourceDesc>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Edge {
    pub start: VertexId,
    pub next: VertexId,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Graph {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
}
