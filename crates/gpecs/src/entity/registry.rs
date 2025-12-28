use std::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    iter::FusedIterator,
    ops::{Index, IndexMut},
};

pub use error::TryReserveError;

pub type TrySpawnError<Meta> = error::TryModifyError<Entity, Meta>;

use gpecs_sparse::{
    arena::EpochSparseArena,
    error,
    iter::{Iter as SparseIter, IterMut as SparseIterMut},
    soa::identity::IdentitySlice,
};

use crate::{soa::identity::Identity, world::registry::WorldId};

use super::{Entity, EntityEpoch};

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
    pub fn as_slice(&self) -> &[Entity] {
        let Self { inner } = self;
        inner.as_key_slice()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const Entity {
        let Self { inner } = self;
        inner.as_keys_ptr()
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
    pub fn iter(&self) -> Iter<'_, '_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter();
        Iter { inner }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, '_, Meta> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        IterMut { inner }
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

impl<'r, Meta> IntoIterator for &'r EntityRegistry<Meta> {
    type Item = (Entity, &'r Meta);
    type IntoIter = Iter<'r, 'r, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'r, Meta> IntoIterator for &'r mut EntityRegistry<Meta> {
    type Item = (Entity, &'r mut Meta);
    type IntoIter = IterMut<'r, 'r, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct Iter<'ctx, 'a, Meta> {
    inner: SparseIter<'ctx, 'a, Entity, Identity<Meta>>,
}

impl<'a, Meta> Iter<'_, 'a, Meta> {
    #[inline]
    pub fn entities(&self) -> &'a [Entity] {
        let (entities, _) = self.as_slices();
        entities
    }

    #[inline]
    pub fn metas(&self) -> &'a [Meta] {
        let (_, metas) = self.as_slices();
        metas
    }

    #[inline]
    pub fn as_slices(&self) -> (&'a [Entity], &'a [Meta]) {
        let Self { inner } = self;

        let (entities, metas) = inner.as_slices();
        let metas = metas.as_inner();
        (entities, metas)
    }
}

impl<Meta> Debug for Iter<'_, '_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, metas) = self.as_slices();
        f.debug_struct("Iter")
            .field("entities", &entities)
            .field("metas", &metas)
            .finish()
    }
}

impl<Meta> Clone for Iter<'_, '_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<Meta> AsRef<[Meta]> for Iter<'_, '_, Meta> {
    #[inline]
    fn as_ref(&self) -> &[Meta] {
        self.metas()
    }
}

impl<'a, Meta> Iterator for Iter<'_, 'a, Meta> {
    type Item = (Entity, &'a Meta);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|(&entity, Identity(meta))| f((entity, meta)));
    }
}

impl<Meta> DoubleEndedIterator for Iter<'_, '_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|(&entity, Identity(meta))| (entity, meta))
    }
}

impl<Meta> ExactSizeIterator for Iter<'_, '_, Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for Iter<'_, '_, Meta> {}

pub struct IterMut<'ctx, 'a, Meta> {
    inner: SparseIterMut<'ctx, 'a, Entity, Identity<Meta>>,
}

impl<'a, Meta> IterMut<'_, 'a, Meta> {
    #[inline]
    pub fn into_entities(self) -> &'a [Entity] {
        let (entities, _) = self.into_slices();
        entities
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        let (entities, _) = self.as_slices();
        entities
    }

    #[inline]
    pub fn into_metas(self) -> &'a mut [Meta] {
        let (_, metas) = self.into_slices();
        metas
    }

    #[inline]
    pub fn metas(&self) -> &[Meta] {
        let (_, metas) = self.as_slices();
        metas
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [Entity], &'a mut [Meta]) {
        let Self { inner } = self;

        let (entities, metas) = inner.into_slices();
        let metas = metas.as_inner_mut();
        (entities, metas)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], &[Meta]) {
        let Self { inner } = self;

        let (entities, metas) = inner.as_slices();
        let metas = metas.as_inner();
        (entities, metas)
    }
}

impl<Meta> Debug for IterMut<'_, '_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, metas) = self.as_slices();
        f.debug_struct("Iter")
            .field("entities", &entities)
            .field("metas", &metas)
            .finish()
    }
}

impl<Meta> AsRef<[Meta]> for IterMut<'_, '_, Meta> {
    #[inline]
    fn as_ref(&self) -> &[Meta] {
        self.metas()
    }
}

impl<'a, Meta> Iterator for IterMut<'_, 'a, Meta> {
    type Item = (Entity, &'a mut Meta);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|(&entity, Identity(meta))| f((entity, meta)));
    }
}

impl<Meta> DoubleEndedIterator for IterMut<'_, '_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|(&entity, Identity(meta))| (entity, meta))
    }
}

impl<Meta> ExactSizeIterator for IterMut<'_, '_, Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for IterMut<'_, '_, Meta> {}
