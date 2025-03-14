use std::{ops::Deref, slice::Iter, vec::IntoIter};

use gpecs_sparse::arena::EpochSparseArena;

use super::Entity;

pub type TryReserveError = gpecs_sparse::error::TryReserveError;

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
        inner.try_reserve(additional, additional)
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { inner } = self;
        inner.try_reserve_exact(additional, additional)
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
    pub fn insert(&mut self, entity: Entity) {
        let Self { inner } = self;
        inner.insert(entity, ());
    }

    #[inline]
    pub fn try_insert(&mut self, entity: Entity) -> Result<(), TryReserveError> {
        let Self { inner } = self;

        inner.try_insert(entity, ())?;
        Ok(())
    }

    #[inline]
    pub fn spawn(&mut self) -> Entity {
        let Self { inner } = self;
        inner.push(())
    }

    #[inline]
    pub fn try_spawn(&mut self) -> Result<Entity, TryReserveError> {
        let Self { inner } = self;
        inner.try_push(())
    }

    #[inline]
    pub fn despawn(&mut self, entity: Entity) {
        let Self { inner } = self;
        inner.remove(entity);
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<usize> {
        let Self { inner } = self;
        inner.get_epoch(sparse_index)
    }

    #[inline]
    pub fn invalidate_epoch(&mut self, entity: Entity) -> Option<Entity> {
        let Self { inner } = self;
        inner.invalidate_epoch(entity)
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

impl From<Vec<Entity>> for EntityRegistry {
    #[inline]
    fn from(value: Vec<Entity>) -> Self {
        value.into_iter().collect()
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

impl FromIterator<Entity> for EntityRegistry {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Entity>>(iter: T) -> Self {
        let inner = iter.into_iter().map(|entity| (entity, ())).collect();
        Self { inner }
    }
}

impl Extend<Entity> for EntityRegistry {
    #[inline]
    fn extend<T: IntoIterator<Item = Entity>>(&mut self, iter: T) {
        let Self { inner } = self;
        inner.extend(iter.into_iter().map(|entity| (entity, ())));
    }
}
