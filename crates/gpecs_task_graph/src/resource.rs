use crate::id::new_id_type;

new_id_type! {
    /// Identifier of a resource accessed by some task.
    pub ResourceId;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceAccess {
    #[default]
    ReadOnly,
    ReadWrite,
    WriteOnly,
}

impl ResourceAccess {
    pub const fn is_read_only(&self) -> bool {
        matches!(self, Self::ReadOnly)
    }

    pub const fn is_read_write(&self) -> bool {
        matches!(self, Self::ReadWrite)
    }

    pub const fn is_write_only(&self) -> bool {
        matches!(self, Self::WriteOnly)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourceDesc {
    pub id: ResourceId,
    pub access: ResourceAccess,
}
