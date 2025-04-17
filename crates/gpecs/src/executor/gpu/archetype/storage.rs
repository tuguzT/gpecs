use crate::archetype::storage::ArchetypeStorage;

#[derive(Debug)]
pub struct GpuArchetypeStorage {
    _inner: (),
    // TODO: store GPU buffer here
    //       also store byte offsets to each component slice of an archetype
}

impl GpuArchetypeStorage {
    #[inline]
    pub fn new(storage: &ArchetypeStorage) -> Self {
        let _ = storage;
        Self { _inner: () }
    }
}
