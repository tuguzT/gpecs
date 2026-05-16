use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
};

use gpecs_sparse::{
    error::FromPartsError,
    item::{KeyValueMutSlices, SparseItem},
    soa::{
        identity::{AsIdentitySlice, Identity, IdentitySlice},
        slice::SoaSlicesMut,
    },
    view::{EpochSparseViewMut, EpochSparseViewMutPtr},
};

use crate::{
    Entity, EntityEpoch, EntitySparseItem,
    registry::{EntityRegistryView, Iter, IterMut},
};

type Inner<'a, Meta, S> = EpochSparseViewMutPtr<'a, Entity, Identity<Meta>, S>;

#[repr(transparent)]
pub struct EntityRegistryViewMut<'a, Meta, S = EntitySparseItem>
where
    Meta: 'a,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + 'a,
{
    inner: Inner<'a, Meta, S>,
}

impl<'a, Meta, S> EntityRegistryViewMut<'a, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    const CONTEXT: &'a () = &();

    #[inline]
    pub fn new(
        entities: &'a mut [Entity],
        metas: &'a mut [Meta],
        sparse: &'a mut [S],
    ) -> Result<Self, FromPartsError<Entity>> {
        let context = Self::CONTEXT;
        let dense = SoaSlicesMut::new(
            Identity::from_inner_ref(context),
            KeyValueMutSlices::new(context, entities, metas.as_identity_slice_mut()),
        );

        let inner = EpochSparseViewMut::new(dense, sparse)?.into_mut_view_ptr();
        let me = Self::from_inner(inner);
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        entities: &'a mut [Entity],
        metas: &'a mut [Meta],
        sparse: &'a mut [S],
    ) -> Self {
        let context = Self::CONTEXT;
        let dense = unsafe {
            SoaSlicesMut::new(
                Identity::from_inner_ref(context),
                KeyValueMutSlices::new_unchecked(entities, metas.as_identity_slice_mut()),
            )
        };

        let inner = unsafe { EpochSparseViewMut::from_parts(dense, sparse) }.into_mut_view_ptr();
        Self::from_inner(inner)
    }

    #[inline]
    pub(super) fn from_inner(inner: Inner<'a, Meta, S>) -> Self {
        Self { inner }
    }

    #[inline]
    #[allow(unused)]
    pub(super) fn into_inner(self) -> Inner<'a, Meta, S> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub unsafe fn into_parts(self) -> (&'a mut [Entity], &'a mut [Meta], &'a mut [S]) {
        let Self { inner } = self;

        let (dense, sparse) = inner.into_mut_slice_ptrs();
        let sparse = unsafe { sparse.as_mut_unchecked() };

        let context = Self::CONTEXT;
        let (entities, metas) = unsafe { dense.as_mut_unchecked(context).into_parts() };
        let metas = metas.as_inner_mut();

        (entities, metas, sparse)
    }

    #[inline]
    pub fn as_view(&self) -> EntityRegistryView<'_, Meta, S> {
        let Self { inner } = self;

        let inner = inner.cast_const();
        EntityRegistryView::from_inner(inner)
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> EntityRegistryViewMut<'_, Meta, S> {
        let Self { inner } = *self;
        EntityRegistryViewMut::from_inner(inner)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], &[Meta], &[S]) {
        let Self { inner } = self;

        let (dense, sparse) = inner.as_slice_ptrs();
        let sparse = unsafe { sparse.as_ref_unchecked() };

        let context = Self::CONTEXT;
        let (entities, metas) = unsafe { dense.as_ref_unchecked(context).into_parts() };
        let metas = metas.as_inner();

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
    pub fn as_sparse(&self) -> &[S] {
        let (_, _, sparse) = self.as_slices();
        sparse
    }

    #[inline]
    pub unsafe fn as_mut_slices(&mut self) -> (&mut [Entity], &mut [Meta], &mut [S]) {
        let Self { inner } = self;

        let (dense, sparse) = inner.as_mut_slice_ptrs();
        let sparse = unsafe { sparse.as_mut_unchecked() };

        let context = Self::CONTEXT;
        let (entities, metas) = unsafe { dense.as_mut_unchecked(context).into_parts() };
        let metas = metas.as_inner_mut();

        (entities, metas, sparse)
    }

    #[inline]
    pub fn as_mut_metas(&mut self) -> &mut [Meta] {
        let (_, metas, _) = unsafe { self.as_mut_slices() };
        metas
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const Entity, *const Meta, *const S) {
        let (entities, metas, sparse) = self.as_slices();
        (entities.as_ptr(), metas.as_ptr(), sparse.as_ptr())
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut Entity, *mut Meta, *mut S) {
        let (entities, metas, sparse) = unsafe { self.as_mut_slices() };
        (
            entities.as_mut_ptr(),
            metas.as_mut_ptr(),
            sparse.as_mut_ptr(),
        )
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { inner } = self;
        unsafe { inner.as_ref_unchecked() }.contains_key(entity)
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&Meta> {
        let Self { inner } = self;

        unsafe { inner.as_ref_unchecked() }
            .into_get(entity)
            .map(Identity::as_inner)
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut Meta> {
        let Self { inner } = self;

        unsafe { inner.as_mut_unchecked() }
            .into_get_mut(entity)
            .map(Identity::as_inner_mut)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<EntityEpoch> {
        let Self { inner } = self;

        let sparse_index = sparse_index.try_into().ok()?;
        unsafe { inner.as_ref_unchecked() }.get_epoch(sparse_index)
    }

    #[inline]
    pub fn invalidate_epoch(&mut self, entity: Entity) -> Option<Entity> {
        let Self { inner } = self;

        let world = entity.world();
        let mut inner = unsafe { inner.as_mut_unchecked() };

        let entity = inner.invalidate_epoch(entity)?;
        let entity = inner
            .replace_key(Entity::new(entity.index(), entity.epoch(), world))
            .expect("entity should exist: it was just created");
        Some(entity)
    }

    #[inline]
    pub fn into_get(self, entity: Entity) -> Option<&'a Meta> {
        let Self { inner } = self;

        unsafe { inner.as_ref_unchecked() }
            .into_get(entity)
            .map(Identity::as_inner)
    }

    #[inline]
    pub fn into_get_mut(self, entity: Entity) -> Option<&'a mut Meta> {
        let Self { inner } = self;

        unsafe { inner.as_mut_unchecked() }
            .into_get_mut(entity)
            .map(Identity::as_inner_mut)
    }

    #[inline]
    pub fn into_index(self, entity: Entity) -> &'a Meta {
        let Self { inner } = self;
        unsafe { inner.as_ref_unchecked() }.into_index(entity)
    }

    #[inline]
    pub fn into_index_mut(self, entity: Entity) -> &'a mut Meta {
        let Self { inner } = self;
        unsafe { inner.as_mut_unchecked() }.into_index_mut(entity)
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

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter(&self) -> crate::registry::ParIter<'_, Meta, S> {
        let view = self.as_view();
        crate::registry::ParIter::new(view)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_mut(&mut self) -> crate::registry::ParIterMut<'_, Meta, S> {
        let view = self.as_mut_view();
        crate::registry::ParIterMut::new(view)
    }
}

impl<Meta, S> Debug for EntityRegistryViewMut<'_, Meta, S>
where
    Meta: Debug,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, metas, sparse) = self.as_slices();
        f.debug_struct("EntityRegistryViewMut")
            .field("entities", &entities)
            .field("metas", &metas)
            .field("sparse", &sparse)
            .finish()
    }
}

impl<Meta, S> Default for EntityRegistryViewMut<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    fn default() -> Self {
        let inner = Inner::from(Self::CONTEXT);
        Self::from_inner(inner)
    }
}

impl<Meta, S> PartialEq for EntityRegistryViewMut<'_, Meta, S>
where
    Meta: PartialEq,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slices() == other.as_slices()
    }
}

impl<Meta, S> Eq for EntityRegistryViewMut<'_, Meta, S>
where
    Meta: Eq,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Eq,
{
}

impl<Meta, S> PartialOrd for EntityRegistryViewMut<'_, Meta, S>
where
    Meta: PartialOrd,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let slices = self.as_slices();
        let other = other.as_slices();
        slices.partial_cmp(&other)
    }
}

impl<Meta, S> Ord for EntityRegistryViewMut<'_, Meta, S>
where
    Meta: Ord,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let slices = self.as_slices();
        let other = other.as_slices();
        slices.cmp(&other)
    }
}

impl<Meta, S> Hash for EntityRegistryViewMut<'_, Meta, S>
where
    Meta: Hash,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slices().hash(state);
    }
}

impl<Meta, S> AsRef<[Entity]> for EntityRegistryViewMut<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    #[inline]
    fn as_ref(&self) -> &[Entity] {
        self.as_entities()
    }
}

impl<Meta, S> Index<Entity> for EntityRegistryViewMut<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    type Output = Meta;

    #[inline]
    fn index(&self, entity: Entity) -> &Self::Output {
        let Self { inner } = self;
        unsafe { inner.as_ref_unchecked() }.into_index(entity)
    }
}

impl<Meta, S> IndexMut<Entity> for EntityRegistryViewMut<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    #[inline]
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        let Self { inner } = self;
        unsafe { inner.as_mut_unchecked() }.into_index_mut(entity)
    }
}

impl<'a, Meta, S> IntoIterator for &'a EntityRegistryViewMut<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    type Item = (Entity, &'a Meta);
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta, S> IntoIterator for &'a mut EntityRegistryViewMut<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    type Item = (Entity, &'a mut Meta);
    type IntoIter = IterMut<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, Meta, S> IntoIterator for EntityRegistryViewMut<'a, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    type Item = (Entity, &'a mut Meta);
    type IntoIter = IterMut<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        IterMut::from_inner(inner)
    }
}

#[cfg(feature = "rayon")]
impl<'a, Meta, S> rayon::iter::IntoParallelIterator for &'a EntityRegistryViewMut<'_, Meta, S>
where
    Meta: Sync,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Sync,
{
    type Item = (Entity, &'a Meta);
    type Iter = crate::registry::ParIter<'a, Meta, S>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, Meta, S> rayon::iter::IntoParallelIterator for &'a mut EntityRegistryViewMut<'_, Meta, S>
where
    Meta: Send,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Send,
{
    type Item = (Entity, &'a mut Meta);
    type Iter = crate::registry::ParIterMut<'a, Meta, S>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter_mut()
    }
}

#[cfg(feature = "rayon")]
impl<'a, Meta, S> rayon::iter::IntoParallelIterator for EntityRegistryViewMut<'a, Meta, S>
where
    Meta: Send,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Send,
{
    type Item = (Entity, &'a mut Meta);
    type Iter = crate::registry::ParIterMut<'a, Meta, S>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        crate::registry::ParIterMut::new(self)
    }
}

unsafe impl<Meta, S> Send for EntityRegistryViewMut<'_, Meta, S>
where
    Meta: Send,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Send,
{
}

unsafe impl<Meta, S> Sync for EntityRegistryViewMut<'_, Meta, S>
where
    Meta: Sync,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Sync,
{
}
