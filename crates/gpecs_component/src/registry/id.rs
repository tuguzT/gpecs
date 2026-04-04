use core::fmt::{self, Debug, Display};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct ComponentId(u32);

impl ComponentId {
    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(id) = self;
        id
    }

    #[inline]
    pub const unsafe fn from_u32(id: u32) -> Self {
        Self(id)
    }
}

impl From<ComponentId> for u32 {
    #[inline]
    fn from(id: ComponentId) -> Self {
        id.into_u32()
    }
}

impl Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(id) = self;
        write!(f, "component {id}")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct GpuComponentId(ComponentId);

impl GpuComponentId {
    #[inline]
    pub const fn into_id(self) -> ComponentId {
        let Self(id) = self;
        id
    }

    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(id) = self;
        id.into_u32()
    }

    #[inline]
    pub const unsafe fn from_id(id: ComponentId) -> Self {
        Self(id)
    }

    #[inline]
    pub const unsafe fn from_u32(id: u32) -> Self {
        let id = unsafe { ComponentId::from_u32(id) };
        Self(id)
    }
}

impl From<GpuComponentId> for u32 {
    #[inline]
    fn from(id: GpuComponentId) -> Self {
        id.into_u32()
    }
}

impl From<GpuComponentId> for ComponentId {
    #[inline]
    fn from(id: GpuComponentId) -> Self {
        id.into_id()
    }
}

impl Debug for GpuComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = &self.into_u32();
        f.debug_tuple("GpuComponentId").field(id).finish()
    }
}

impl Display for GpuComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(id) = self;

        if !f.alternate() {
            write!(f, "GPU ")?;
        }
        Display::fmt(id, f)
    }
}
