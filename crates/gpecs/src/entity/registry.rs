use std::{
    fmt::{self, Debug},
    hash::{self, Hash},
    num::Wrapping,
    ops::Deref,
    slice::Iter,
};

pub use error::TryReserveError;

pub type EntityOverflowError = error::InvalidKeyError<Entity>;
pub type TryEntityOverflowError = error::TryInvalidKeyError<Entity>;

use gpecs_sparse::{arena::EpochSparseArena, error};

use crate::{soa::traits::Soa, world::registry::WorldId};

use super::Entity;

pub struct EntityRegistry<Meta = ()>
where
    Meta: Soa,
{
    inner: EpochSparseArena<Entity, Meta>,
}

impl<Meta> EntityRegistry<Meta>
where
    Meta: Soa,
{
    #[inline]
    pub fn new() -> Self
    where
        Meta::Context: Default,
    {
        let inner = EpochSparseArena::new();
        Self { inner }
    }

    #[inline]
    pub fn with_context(context: Meta::Context) -> Self {
        let inner = EpochSparseArena::with_context(context);
        Self { inner }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self
    where
        Meta::Context: Default,
    {
        let inner = EpochSparseArena::with_capacity(capacity, capacity);
        Self { inner }
    }

    #[inline]
    pub fn with_context_and_capacity(context: Meta::Context, capacity: usize) -> Self {
        let inner = EpochSparseArena::with_context_and_capacity(context, capacity, capacity);
        Self { inner }
    }

    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError>
    where
        Meta::Context: Default,
    {
        let inner = EpochSparseArena::try_with_capacity(capacity, capacity)?;
        Ok(Self { inner })
    }

    #[inline]
    pub fn try_with_context_and_capacity(
        context: Meta::Context,
        capacity: usize,
    ) -> Result<Self, TryReserveError> {
        let inner = EpochSparseArena::try_with_context_and_capacity(context, capacity, capacity)?;
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
    pub fn insert(
        &mut self,
        entity: Entity,
        meta: Meta,
    ) -> Result<Option<Meta>, EntityOverflowError> {
        let Self { inner } = self;
        inner.insert(entity, meta)
    }

    #[inline]
    pub fn try_insert(
        &mut self,
        entity: Entity,
        meta: Meta,
    ) -> Result<Option<Meta>, TryEntityOverflowError> {
        let Self { inner } = self;
        inner.try_insert(entity, meta)
    }

    #[inline]
    pub fn spawn(&mut self, world: WorldId, meta: Meta) -> Result<Entity, EntityOverflowError> {
        let Self { inner } = self;

        let entity = inner.push(meta)?;
        let entity = inner
            .replace_key(Entity::new(entity.index(), entity.epoch(), world))
            .expect("entity should exist because it was just created");
        Ok(entity)
    }

    #[inline]
    pub fn try_spawn(
        &mut self,
        world: WorldId,
        meta: Meta,
    ) -> Result<Entity, TryEntityOverflowError> {
        let Self { inner } = self;

        let entity = inner.try_push(meta)?;
        let entity = inner
            .replace_key(Entity::new(entity.index(), entity.epoch(), world))
            .expect("entity should exist because it was just created");
        Ok(entity)
    }

    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> Option<Meta> {
        let Self { inner } = self;
        inner.remove(entity)
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
    pub fn get(&self, entity: Entity) -> Option<Meta::Refs<'_>> {
        let Self { inner } = self;
        inner.get(entity)
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<Meta::RefsMut<'_>> {
        let Self { inner } = self;
        inner.get_mut(entity)
    }

    #[inline]
    pub fn index(&self, entity: Entity) -> Meta::Refs<'_> {
        let Self { inner } = self;
        inner.index(entity)
    }

    #[inline]
    pub fn index_mut(&mut self, entity: Entity) -> Meta::RefsMut<'_> {
        let Self { inner } = self;
        inner.index_mut(entity)
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

impl<Meta> Debug for EntityRegistry<Meta>
where
    Meta: Soa,
    EpochSparseArena<Entity, Meta>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("EntityRegistry")
            .field("inner", inner)
            .finish()
    }
}

impl<Meta> Default for EntityRegistry<Meta>
where
    Meta: Soa,
    Meta::Context: Default,
{
    fn default() -> Self {
        Self {
            inner: EpochSparseArena::new(),
        }
    }
}

impl<Meta> PartialEq for EntityRegistry<Meta>
where
    Meta: Soa,
    EpochSparseArena<Entity, Meta>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner } = self;
        *inner == other.inner
    }
}

impl<Meta> Eq for EntityRegistry<Meta>
where
    Meta: Soa,
    EpochSparseArena<Entity, Meta>: Eq,
{
}

impl<Meta> PartialOrd for EntityRegistry<Meta>
where
    Meta: Soa,
    EpochSparseArena<Entity, Meta>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let Self { inner } = self;
        inner.partial_cmp(&other.inner)
    }
}

impl<Meta> Ord for EntityRegistry<Meta>
where
    Meta: Soa,
    EpochSparseArena<Entity, Meta>: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let Self { inner } = self;
        inner.cmp(&other.inner)
    }
}

impl<Meta> Hash for EntityRegistry<Meta>
where
    Meta: Soa,
    EpochSparseArena<Entity, Meta>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner } = self;
        inner.hash(state);
    }
}

impl<Meta> Clone for EntityRegistry<Meta>
where
    Meta: Soa,
    EpochSparseArena<Entity, Meta>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let Self { inner: this } = self;
        let Self { inner: source } = source;
        this.clone_from(source);
    }
}

impl<Meta> AsRef<[Entity]> for EntityRegistry<Meta>
where
    Meta: Soa,
{
    #[inline]
    fn as_ref(&self) -> &[Entity] {
        self.as_slice()
    }
}

impl<Meta> AsRef<EntityRegistry<Meta>> for EntityRegistry<Meta>
where
    Meta: Soa,
{
    #[inline]
    fn as_ref(&self) -> &EntityRegistry<Meta> {
        self
    }
}

impl<Meta> AsMut<EntityRegistry<Meta>> for EntityRegistry<Meta>
where
    Meta: Soa,
{
    #[inline]
    fn as_mut(&mut self) -> &mut EntityRegistry<Meta> {
        self
    }
}

impl<Meta> Deref for EntityRegistry<Meta>
where
    Meta: Soa,
{
    type Target = [Entity];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a, Meta> IntoIterator for &'a EntityRegistry<Meta>
where
    Meta: Soa,
{
    type Item = &'a Entity;
    type IntoIter = Iter<'a, Entity>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
