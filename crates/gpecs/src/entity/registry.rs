use std::{num::Wrapping, ops::Deref, slice::Iter, vec::IntoIter};

pub use error::TryReserveError;

pub type EntityOverflowError = error::InvalidKeyError<Entity>;
pub type TryEntityOverflowError = error::TryInvalidKeyError<Entity>;

use gpecs_sparse::{arena::EpochSparseArena, error};

use crate::world::registry::WorldId;

use super::Entity;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityRegistry {
    inner: EpochSparseArena<Entity, ()>,
}

impl EntityRegistry {
    #[inline]
    pub fn new() -> Self {
        let inner = EpochSparseArena::new();
        Self { inner }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let inner = EpochSparseArena::with_capacity(capacity, capacity);
        Self { inner }
    }

    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let inner = EpochSparseArena::try_with_capacity(capacity, capacity)?;
        Ok(Self { inner })
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { inner } = self;
        inner.is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner } = self;
        inner.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let Self { inner } = self;
        inner.reserve(additional, additional);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        let Self { inner } = self;
        inner.reserve_exact(additional, additional);
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { inner } = self;

        inner.try_reserve(additional, additional)?;
        Ok(())
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { inner } = self;

        inner.try_reserve_exact(additional, additional)?;
        Ok(())
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        let Self { inner } = self;
        inner.dense_shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let Self { inner } = self;
        inner.dense_shrink_to(min_capacity);
    }

    #[inline]
    pub fn as_slice(&self) -> &[Entity] {
        let Self { inner } = self;
        inner.as_keys_slice()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const Entity {
        let Self { inner } = self;
        inner.as_keys_ptr()
    }

    #[inline]
    pub fn insert(&mut self, entity: Entity) -> Result<(), EntityOverflowError> {
        let Self { inner } = self;

        inner.insert(entity, ())?;
        Ok(())
    }

    #[inline]
    pub fn try_insert(&mut self, entity: Entity) -> Result<(), TryEntityOverflowError> {
        let Self { inner } = self;

        inner.try_insert(entity, ())?;
        Ok(())
    }

    #[inline]
    pub fn spawn(&mut self, world: WorldId) -> Result<Entity, EntityOverflowError> {
        let Self { inner } = self;

        let entity = inner.push(())?;
        let entity = inner
            .replace_key(Entity::new(entity.index(), entity.epoch(), world))
            .expect("entity should exist because it was just created");
        Ok(entity)
    }

    #[inline]
    pub fn try_spawn(&mut self, world: WorldId) -> Result<Entity, TryEntityOverflowError> {
        let Self { inner } = self;

        let entity = inner.try_push(())?;
        let entity = inner
            .replace_key(Entity::new(entity.index(), entity.epoch(), world))
            .expect("entity should exist because it was just created");
        Ok(entity)
    }

    #[inline]
    pub fn despawn(&mut self, entity: Entity) {
        let Self { inner } = self;
        inner.remove(entity);
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: u32) -> Option<u16> {
        let Self { inner } = self;
        inner.get_epoch(sparse_index).map(|Wrapping(epoch)| epoch)
    }

    #[inline]
    pub fn invalidate_epoch(&mut self, entity: Entity) -> Option<Entity> {
        let Self { inner } = self;

        let world = entity.world();
        let entity = inner.invalidate_epoch(entity)?;
        let entity = inner
            .replace_key(Entity::new(entity.index(), entity.epoch(), world))
            .expect("entity should exist: it was just created");
        Some(entity)
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        let Self { inner } = self;
        inner.truncate(len, usize::MAX);
    }

    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(Entity) -> bool,
    {
        let Self { inner } = self;
        inner.retain(|entity, _| f(entity));
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { inner } = self;
        inner.contains_key(entity)
    }

    #[inline]
    pub fn clear(&mut self) {
        let Self { inner } = self;
        inner.clear();
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Entity> {
        self.as_slice().iter()
    }
}

impl From<EntityRegistry> for Vec<Entity> {
    #[inline]
    fn from(storage: EntityRegistry) -> Self {
        let EntityRegistry { inner } = storage;
        inner.into_keys_vec()
    }
}

impl Clone for EntityRegistry {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let Self { inner: this } = self;
        let Self { inner: source } = source;
        this.clone_from(source);
    }
}

impl AsRef<[Entity]> for EntityRegistry {
    #[inline]
    fn as_ref(&self) -> &[Entity] {
        self.as_slice()
    }
}

impl AsRef<EntityRegistry> for EntityRegistry {
    #[inline]
    fn as_ref(&self) -> &EntityRegistry {
        self
    }
}

impl AsMut<EntityRegistry> for EntityRegistry {
    #[inline]
    fn as_mut(&mut self) -> &mut EntityRegistry {
        self
    }
}

impl Deref for EntityRegistry {
    type Target = [Entity];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a> IntoIterator for &'a EntityRegistry {
    type Item = &'a Entity;
    type IntoIter = Iter<'a, Entity>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for EntityRegistry {
    type Item = Entity;
    type IntoIter = IntoIter<Entity>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let vec: Vec<_> = self.into();
        vec.into_iter()
    }
}
