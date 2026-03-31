use core::{
    fmt::{self, Debug},
    hash::{BuildHasher, Hash},
};

use hashbrown::{DefaultHashBuilder, HashMap};

use crate::registry::{
    ComponentId,
    traits::{ComponentIdFrom, ComponentIdFromOrInsertWith},
};

#[derive(Clone)]
pub struct ComponentIdMap<K, S = DefaultHashBuilder> {
    map: HashMap<K, ComponentId, S>,
}

impl<K, S> ComponentIdMap<K, S> {
    #[inline]
    pub const fn with_hasher(hash_builder: S) -> Self {
        let map = HashMap::with_hasher(hash_builder);
        Self { map }
    }

    #[inline]
    pub fn into_inner(self) -> HashMap<K, ComponentId, S> {
        let Self { map } = self;
        map
    }
}

impl<K, S> ComponentIdMap<K, S>
where
    S: Default,
{
    #[inline]
    pub fn new() -> Self {
        let hash_builder = S::default();
        Self::with_hasher(hash_builder)
    }
}

impl<K, S> Debug for ComponentIdMap<K, S>
where
    K: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { map } = self;
        f.debug_struct("ComponentIdMap").field("map", map).finish()
    }
}

impl<K, S> Default for ComponentIdMap<K, S>
where
    S: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, S> PartialEq for ComponentIdMap<K, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { map } = self;
        let Self { map: other } = other;
        *map == *other
    }
}

impl<K, S> Eq for ComponentIdMap<K, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
}

unsafe impl<K, S> ComponentIdFrom for ComponentIdMap<K, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    type Key = K;

    #[inline]
    fn component_id_from(&self, key: Self::Key) -> Option<ComponentId> {
        let Self { map, .. } = self;
        map.get(&key).copied()
    }
}

unsafe impl<K, S> ComponentIdFromOrInsertWith for ComponentIdMap<K, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    fn component_id_from_or_insert_with<F>(&mut self, key: Self::Key, f: F) -> ComponentId
    where
        F: FnOnce() -> ComponentId,
    {
        let Self { map } = self;
        *map.entry(key).or_insert_with(f)
    }
}
