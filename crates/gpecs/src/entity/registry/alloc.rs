use std::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
};

pub use error::TryReserveError;

use gpecs_sparse::{arena::EpochSparseArena, error};

use crate::{
    entity::{Entity, EntityEpoch},
    soa::identity::{Identity, IdentityMutPtr, IdentityPtr, IdentitySlice},
    world::id::WorldId,
};

use super::{EntityRegistryView, Iter, IterMut};

pub type TrySpawnError<Meta> = error::TryModifyError<Entity, Meta>;

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
    #[must_use]
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
    pub fn as_view(&self) -> EntityRegistryView<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.as_view();
        EntityRegistryView::from_inner(inner)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], &[Meta]) {
        let Self { inner } = self;

        let (entities, metas) = inner.as_dense_slices().into_parts();
        let metas = metas.as_inner();
        (entities, metas)
    }

    #[inline]
    pub fn as_entities(&self) -> &[Entity] {
        let (entities, _) = self.as_slices();
        entities
    }

    #[inline]
    pub fn as_metas(&self) -> &[Meta] {
        let (_, metas) = self.as_slices();
        metas
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&[Entity], &mut [Meta]) {
        let Self { inner } = self;

        let (entities, metas) = unsafe { inner.as_mut_dense_slices().into_parts() };
        let metas = metas.as_inner_mut();
        (entities, metas)
    }

    #[inline]
    pub fn as_mut_metas(&mut self) -> &mut [Meta] {
        let (_, metas) = self.as_mut_slices();
        metas
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const Entity, *const Meta) {
        let Self { inner } = self;

        let (entities, metas) = inner.as_dense_ptrs().into_parts();
        let metas = metas.as_inner_ptr();
        (entities, metas)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*const Entity, *mut Meta) {
        let Self { inner } = self;

        let (entities, metas) = inner.as_mut_dense_ptrs().into_parts();
        let metas = metas.as_inner_mut_ptr();
        (entities, metas)
    }

    #[inline]
    #[track_caller]
    pub fn spawn(&mut self, world: WorldId, meta: Meta) -> Entity {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn spawn_failed<Meta>(error: TrySpawnError<Meta>) -> ! {
            let kind = error.kind;
            panic!("failed to spawn entity: {kind}")
        }

        self.try_spawn(world, meta)
            .unwrap_or_else(|error| spawn_failed(error))
    }

    #[inline]
    pub fn try_spawn(&mut self, world: WorldId, meta: Meta) -> Result<Entity, TrySpawnError<Meta>> {
        let Self { inner } = self;

        let entity = inner
            .try_push(meta.into())
            .map_err(|error| error.map_value(Identity::into_inner))?;
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
    pub fn get_epoch(&self, sparse_index: u32) -> Option<EntityEpoch> {
        let Self { inner } = self;
        inner.get_epoch(sparse_index)
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
        inner.clear();
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter();
        Iter::from_inner(inner)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        IterMut::from_inner(inner)
    }
}

impl<Meta> Debug for EntityRegistry<Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, metas) = self.as_slices();
        f.debug_struct("EntityRegistry")
            .field("entities", &entities)
            .field("metas", &metas)
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
    Meta: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slices() == other.as_slices()
    }
}

impl<Meta> Eq for EntityRegistry<Meta> where Meta: Eq {}

impl<Meta> PartialOrd for EntityRegistry<Meta>
where
    Meta: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let other = other.as_slices();
        self.as_slices().partial_cmp(&other)
    }
}

impl<Meta> Ord for EntityRegistry<Meta>
where
    Meta: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let other = other.as_slices();
        self.as_slices().cmp(&other)
    }
}

impl<Meta> Hash for EntityRegistry<Meta>
where
    Meta: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slices().hash(state);
    }
}

impl<Meta> Clone for EntityRegistry<Meta>
where
    Meta: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let Self { inner } = self;

        let Self { inner: source } = source;
        inner.clone_from(source);
    }
}

impl<Meta> AsRef<Self> for EntityRegistry<Meta> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<Meta> AsRef<[Entity]> for EntityRegistry<Meta> {
    #[inline]
    fn as_ref(&self) -> &[Entity] {
        self.as_entities()
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

    #[inline]
    fn index(&self, index: Entity) -> &Self::Output {
        let Self { inner } = self;
        let Identity(meta) = inner.index(index);
        meta
    }
}

impl<Meta> IndexMut<Entity> for EntityRegistry<Meta> {
    #[inline]
    fn index_mut(&mut self, index: Entity) -> &mut Self::Output {
        let Self { inner } = self;
        let Identity(meta) = inner.index_mut(index);
        meta
    }
}

impl<'a, Meta> IntoIterator for &'a EntityRegistry<Meta> {
    type Item = (Entity, &'a Meta);
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta> IntoIterator for &'a mut EntityRegistry<Meta> {
    type Item = (Entity, &'a mut Meta);
    type IntoIter = IterMut<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
