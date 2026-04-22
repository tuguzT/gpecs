use std::fmt::{self, Display};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct SystemId(u32);

impl SystemId {
    #[inline]
    pub const fn into_u32(&self) -> u32 {
        let Self(id) = *self;
        id
    }

    #[inline]
    pub const unsafe fn from_u32(id: u32) -> Self {
        Self(id)
    }
}

impl From<SystemId> for u32 {
    #[inline]
    fn from(id: SystemId) -> Self {
        id.into_u32()
    }
}

impl Display for SystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(id) = self;
        write!(f, "system {id}")
    }
}

#[inline]
pub fn system_id_from_usize(index: usize) -> SystemId {
    let id = index.try_into().expect("`SystemId` overflow");
    system_id_trusted(id)
}

#[inline]
pub fn system_id_into_usize(id: SystemId) -> usize {
    let id = id.into_u32();
    id.try_into().expect("`SystemId` overflow")
}

#[inline]
pub fn system_id_trusted(id: u32) -> SystemId {
    unsafe { SystemId::from_u32(id) }
}
