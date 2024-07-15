//! Nothing too special, too =)

use id::id;

mod id;

id!(pub ResourceId);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceAccess {
    #[default]
    ReadOnly,
    ReadWrite,
    WriteOnly,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourceDesc {
    pub id: ResourceId,
    pub access: ResourceAccess,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Vertex {
    pub inputs: Vec<ResourceDesc>,
    pub outputs: Vec<ResourceDesc>,
}
