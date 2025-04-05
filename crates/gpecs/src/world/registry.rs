#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct WorldId(u16);

impl WorldId {
    #[inline]
    pub const fn index(&self) -> u16 {
        let Self(id) = *self;
        id
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
    next_id: u16,
    len: u16,
}

impl WorldRegistry {
    #[inline]
    pub const fn new() -> Self {
        Self { next_id: 1, len: 1 }
    }

    #[inline]
    pub const fn spawn(&mut self) -> WorldId {
        let Self { next_id, len } = self;

        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        *len = len.saturating_add(1);
        WorldId(id)
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

impl Default for WorldRegistry {
    fn default() -> Self {
        Self::new()
    }
}
