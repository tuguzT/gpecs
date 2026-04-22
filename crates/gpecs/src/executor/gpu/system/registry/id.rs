use std::fmt::{self, Display};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct GpuSystemId(u32);

impl GpuSystemId {
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

impl From<GpuSystemId> for u32 {
    #[inline]
    fn from(id: GpuSystemId) -> Self {
        id.into_u32()
    }
}

impl Display for GpuSystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(id) = self;

        if !f.alternate() {
            write!(f, "GPU ")?;
        }
        write!(f, "system {id}")
    }
}

#[inline]
pub fn gpu_system_id_from_usize(index: usize) -> GpuSystemId {
    let id = index.try_into().expect("`GpuSystemId` overflow");
    gpu_system_id_trusted(id)
}

#[inline]
pub fn gpu_system_id_into_usize(id: GpuSystemId) -> usize {
    let id = id.into_u32();
    id.try_into().expect("`GpuSystemId` overflow")
}

#[inline]
pub fn gpu_system_id_trusted(id: u32) -> GpuSystemId {
    unsafe { GpuSystemId::from_u32(id) }
}
