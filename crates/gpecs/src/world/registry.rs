use std::{
    fmt::{self, Debug},
    num::NonZeroU16,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct WorldId(Option<NonZeroU16>);

impl WorldId {
    pub const NULL_NAME: &str = "WorldIdNull";

    #[inline]
    pub const fn null() -> Self {
        Self(None)
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        let Self(id) = self;
        id.is_none()
    }

    #[inline]
    pub const fn index(&self) -> u16 {
        let Self(id) = *self;
        match id {
            Some(id) => id.get(),
            None => 0,
        }
    }
}

impl Debug for WorldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            return f.write_str(Self::NULL_NAME);
        }

        let index = &self.index();
        f.debug_tuple("WorldId").field(index).finish()
    }
}

impl Default for WorldId {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

impl From<WorldId> for u16 {
    #[inline]
    fn from(value: WorldId) -> Self {
        value.index()
    }
}

#[derive(Debug, Clone)]
pub struct WorldRegistry {
    next_id: NonZeroU16,
    len: u16,
}

impl WorldRegistry {
    #[inline]
    pub const fn new() -> Self {
        Self {
            next_id: NonZeroU16::MIN,
            len: 0,
        }
    }

    #[inline]
    pub const fn spawn(&mut self) -> WorldId {
        let Self { next_id, len } = self;

        let id = *next_id;
        *next_id = wrapping_inc(*next_id);
        *len = len.saturating_add(1);
        WorldId(Some(id))
    }

    #[inline]
    pub const fn len(&self) -> u16 {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[inline]
const fn wrapping_inc(value: NonZeroU16) -> NonZeroU16 {
    match value.checked_add(1) {
        Some(value) => value,
        None => NonZeroU16::MIN,
    }
}
