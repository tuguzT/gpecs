use std::{any::TypeId, collections::HashMap};

use crate::{
    component::registry::{
        ComponentId,
        traits::{ComponentIdFrom, ComponentIdFromOrInsertWith},
    },
    hash::BuildHasher,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ComponentTypeIdMap {
    map: HashMap<TypeId, ComponentId, BuildHasher>,
}

impl ComponentIdFrom for ComponentTypeIdMap {
    type Key = TypeId;

    #[inline]
    fn component_id_from(&self, key: Self::Key) -> Option<ComponentId> {
        let Self { map, .. } = self;
        map.get(&key).copied()
    }
}

impl ComponentIdFromOrInsertWith for ComponentTypeIdMap {
    #[inline]
    fn component_id_from_or_insert_with<F>(&mut self, key: Self::Key, f: F) -> ComponentId
    where
        F: FnOnce() -> ComponentId,
    {
        let Self { map } = self;
        *map.entry(key).or_insert_with(f)
    }
}
