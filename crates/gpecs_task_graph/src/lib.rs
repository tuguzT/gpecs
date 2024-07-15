//! Nothing too special, too =)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ResourceId(u32);

impl ResourceId {
    pub const fn empty() -> Self {
        Self(u32::MAX)
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<u32> for ResourceId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<ResourceId> for u32 {
    fn from(value: ResourceId) -> Self {
        let ResourceId(value) = value;
        value
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceAccess {
    #[default]
    ReadOnly,
    ReadWrite,
    WriteOnly,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Vertex {
    pub inputs: Vec<VertexResource>,
    pub outputs: Vec<VertexResource>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VertexResource {
    pub id: ResourceId,
    pub access: ResourceAccess,
}
