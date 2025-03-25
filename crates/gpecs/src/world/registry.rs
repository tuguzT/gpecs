#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct WorldId(u16);

impl WorldId {
    #[inline]
    pub fn index(&self) -> u16 {
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

#[derive(Debug, Default, Clone)]
pub struct WorldRegistry {
    next_id: u16,
    max_id: u16,
}

impl WorldRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            next_id: 0,
            max_id: 0,
        }
    }

    #[inline]
    pub fn create(&mut self) -> WorldId {
        let Self { next_id, max_id } = self;

        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        *max_id = max_id.saturating_add(1);
        WorldId(id)
    }

    #[inline]
    pub fn len(&self) -> u16 {
        let Self { max_id, .. } = *self;
        max_id
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
