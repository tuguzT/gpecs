use std::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    num::Wrapping,
    ops::{Index, IndexMut},
};

pub use error::TryReserveError;

pub type TrySpawnError = error::TryModifyError<Entity>;

use gpecs_sparse::{arena::EpochSparseArena, error};

use crate::{soa::identity::Identity, world::registry::WorldId};

use super::Entity;

pub struct EntityRegistry<Meta = ()> {
    inner: EpochSparseArena<Entity, Identity<Meta>>,
}

impl<Meta> EntityRegistry<Meta> {
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
        inner.reserve(additional, additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        let Self { inner } = self;
        inner.reserve_exact(additional, additional)
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
        inner.dense_shrink_to_fit()
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let Self { inner } = self;
        inner.dense_shrink_to(min_capacity)
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
    #[track_caller]
    pub fn spawn(&mut self, world: WorldId, meta: Meta) -> Entity {
        match self.try_spawn(world, meta) {
            Ok(entity) => entity,
            Err(error) => panic!("failed to spawn entity: {error}"),
        }
    }

    #[inline]
    pub fn try_spawn(&mut self, world: WorldId, meta: Meta) -> Result<Entity, TrySpawnError> {
        let Self { inner } = self;

        let entity = inner.try_push(meta.into())?;
        let entity = inner
            .replace_key(Entity::new(entity.index(), entity.epoch(), world))
            .expect("entity should exist because it was just created");
        Ok(entity)
    }

    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> Option<Meta> {
        let Self { inner } = self;
        inner.swap_remove(entity).map(Identity::into_inner)
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
    pub fn get(&self, entity: Entity) -> Option<&Meta> {
        let Self { inner } = self;

        let Identity(meta) = inner.get(entity)?;
        Some(meta)
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut Meta> {
        let Self { inner } = self;

        let Identity(meta) = inner.get_mut(entity)?;
        Some(meta)
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { inner } = self;
        inner.contains_key(entity)
    }

    #[inline]
    pub fn clear(&mut self) {
        let Self { inner } = self;
        inner.clear()
    }
}

impl<Meta> Debug for EntityRegistry<Meta>
where
    EpochSparseArena<Entity, Identity<Meta>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("EntityRegistry")
            .field("inner", inner)
            .finish()
    }
}

impl<Meta> Default for EntityRegistry<Meta> {
    fn default() -> Self {
        let inner = EpochSparseArena::default();
        Self { inner }
    }
}

impl<Meta> PartialEq for EntityRegistry<Meta>
where
    EpochSparseArena<Entity, Identity<Meta>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner } = self;
        *inner == other.inner
    }
}

impl<Meta> Eq for EntityRegistry<Meta> where EpochSparseArena<Entity, Identity<Meta>>: Eq {}

impl<Meta> PartialOrd for EntityRegistry<Meta>
where
    EpochSparseArena<Entity, Identity<Meta>>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner } = self;
        inner.partial_cmp(&other.inner)
    }
}

impl<Meta> Ord for EntityRegistry<Meta>
where
    EpochSparseArena<Entity, Identity<Meta>>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner } = self;
        inner.cmp(&other.inner)
    }
}

impl<Meta> Hash for EntityRegistry<Meta>
where
    EpochSparseArena<Entity, Identity<Meta>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner } = self;
        inner.hash(state);
    }
}

impl<Meta> Clone for EntityRegistry<Meta>
where
    EpochSparseArena<Entity, Identity<Meta>>: Clone,
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

impl<Meta> AsRef<[Entity]> for EntityRegistry<Meta> {
    #[inline]
    fn as_ref(&self) -> &[Entity] {
        self.as_slice()
    }
}

impl<Meta> AsRef<Self> for EntityRegistry<Meta> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<Meta> AsMut<Self> for EntityRegistry<Meta> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<Meta> Index<Entity> for EntityRegistry<Meta> {
    type Output = Meta;

    fn index(&self, index: Entity) -> &Self::Output {
        let Self { inner } = self;
        let Identity(meta) = inner.index(index);
        meta
    }
}

impl<Meta> IndexMut<Entity> for EntityRegistry<Meta> {
    fn index_mut(&mut self, index: Entity) -> &mut Self::Output {
        let Self { inner } = self;
        let Identity(meta) = inner.index_mut(index);
        meta
    }
}
