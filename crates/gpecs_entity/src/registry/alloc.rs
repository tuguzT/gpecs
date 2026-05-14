use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
};

pub use error::TryReserveError;

use gpecs_sparse::{
    arena::EpochSparseArena, error, item::DefaultSparseItem, soa::identity::Identity,
};
use gpecs_world::id::WorldId;

use crate::entity::{Entity, EntityEpoch};

use super::{EntityRegistryView, EntityRegistryViewMut, Iter, IterMut};

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

        let inner = inner.as_view_ptr();
        EntityRegistryView::from_inner(inner)
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> EntityRegistryViewMut<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.as_mut_view_ptr();
        EntityRegistryViewMut::from_inner(inner)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], &[Meta], &[DefaultSparseItem<Entity>]) {
        let (entities, metas, sparse) = self.as_view().into_parts();
        (entities, metas, sparse)
    }

    #[inline]
    pub fn as_entities(&self) -> &[Entity] {
        let (entities, _, _) = self.as_slices();
        entities
    }

    #[inline]
    pub fn as_metas(&self) -> &[Meta] {
        let (_, metas, _) = self.as_slices();
        metas
    }

    #[inline]
    pub fn as_sparse(&self) -> &[DefaultSparseItem<Entity>] {
        let (_, _, sparse) = self.as_slices();
        sparse
    }

    #[inline]
    pub unsafe fn as_mut_slices(
        &mut self,
    ) -> (&mut [Entity], &mut [Meta], &mut [DefaultSparseItem<Entity>]) {
        let (entities, metas, sparse) = unsafe { self.as_mut_view().into_parts() };
        (entities, metas, sparse)
    }

    #[inline]
    pub fn as_mut_metas(&mut self) -> &mut [Meta] {
        let (_, metas, _) = unsafe { self.as_mut_slices() };
        metas
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const Entity, *const Meta, *const DefaultSparseItem<Entity>) {
        self.as_view().as_ptrs()
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut Entity, *mut Meta, *mut DefaultSparseItem<Entity>) {
        self.as_mut_view().as_mut_ptrs()
    }

    #[inline]
    #[track_caller]
    pub fn spawn(&mut self, world: WorldId, meta: Meta) -> Entity {
        #[cold]
        #[inline(never)]
        #[track_caller]
        #[expect(clippy::needless_pass_by_value)]
        fn spawn_failed(error: error::TryModifyErrorKind<Entity>) -> ! {
            panic!("failed to spawn entity: {error}")
        }

        self.try_spawn(world, meta)
            .map_err(TrySpawnError::into_source)
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
    pub fn get_epoch(&self, sparse_index: usize) -> Option<EntityEpoch> {
        self.as_view().get_epoch(sparse_index)
    }

    #[inline]
    pub fn invalidate_epoch(&mut self, entity: Entity) -> Option<Entity> {
        self.as_mut_view().invalidate_epoch(entity)
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        let Self { inner } = self;
        inner.truncate(len, usize::MAX);
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&Meta> {
        self.as_view().into_get(entity)
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut Meta> {
        self.as_mut_view().into_get_mut(entity)
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.as_view().contains(entity)
    }

    #[inline]
    pub fn clear(&mut self) {
        let Self { inner } = self;
        inner.clear();
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Meta> {
        self.as_view().into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, Meta> {
        self.as_mut_view().into_iter()
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter(&self) -> crate::registry::ParIter<'_, Meta> {
        let view = self.as_view();
        crate::registry::ParIter::new(view)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_mut(&mut self) -> crate::registry::ParIterMut<'_, Meta> {
        let view = self.as_mut_view();
        crate::registry::ParIterMut::new(view)
    }
}

impl<Meta> Debug for EntityRegistry<Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, metas, sparse) = self.as_slices();
        f.debug_struct("EntityRegistry")
            .field("entities", &entities)
            .field("metas", &metas)
            .field("sparse", &sparse)
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
        self.as_view() == other.as_view()
    }
}

impl<Meta> Eq for EntityRegistry<Meta> where Meta: Eq {}

impl<Meta> PartialOrd for EntityRegistry<Meta>
where
    Meta: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let other = other.as_view();
        self.as_view().partial_cmp(&other)
    }
}

impl<Meta> Ord for EntityRegistry<Meta>
where
    Meta: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let other = other.as_view();
        self.as_view().cmp(&other)
    }
}

impl<Meta> Hash for EntityRegistry<Meta>
where
    Meta: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_view().hash(state);
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
    fn index(&self, entity: Entity) -> &Self::Output {
        self.as_view().into_index(entity)
    }
}

impl<Meta> IndexMut<Entity> for EntityRegistry<Meta> {
    #[inline]
    fn index_mut(&mut self, index: Entity) -> &mut Self::Output {
        self.as_mut_view().into_index_mut(index)
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

#[cfg(feature = "rayon")]
impl<'a, Meta> rayon::iter::IntoParallelIterator for &'a EntityRegistry<Meta>
where
    Meta: Sync,
{
    type Item = (Entity, &'a Meta);
    type Iter = crate::registry::ParIter<'a, Meta>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, Meta> rayon::iter::IntoParallelIterator for &'a mut EntityRegistry<Meta>
where
    Meta: Send,
{
    type Item = (Entity, &'a mut Meta);
    type Iter = crate::registry::ParIterMut<'a, Meta>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter_mut()
    }
}
