#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    #[inline]
    pub const fn into_inner(self) -> u32 {
        let Self(id) = self;
        id
    }

    #[inline]
    #[allow(unsafe_code)]
    pub const unsafe fn from_inner(id: u32) -> Self {
        Self(id)
    }
}
