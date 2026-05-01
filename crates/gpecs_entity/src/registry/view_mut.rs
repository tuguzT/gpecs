use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
};

use gpecs_sparse::{
    error::FromPartsError,
    item::{DenseSlicesMut, SparseItem},
    soa::{
        identity::{AsIdentitySlice, Identity, IdentitySlice},
        slice::SoaSlicesMut,
    },
    view::{EpochSparseViewMut, EpochSparseViewMutPtr},
};

use crate::{
    entity::{Entity, EntityEpoch},
    registry::{EntityRegistryView, Iter, IterMut},
};

type Inner<'a, Meta> = EpochSparseViewMutPtr<'a, Entity, Identity<Meta>>;

#[repr(transparent)]
pub struct EntityRegistryViewMut<'a, Meta>
where
    Meta: 'a,
{
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> EntityRegistryViewMut<'a, Meta> {
    const CONTEXT: &'a () = &();

    #[inline]
    pub fn new(
        entities: &'a mut [Entity],
        metas: &'a mut [Meta],
        sparse: &'a mut [SparseItem<Entity>],
    ) -> Result<Self, FromPartsError<Entity>> {
        let context = Self::CONTEXT;
        let dense = SoaSlicesMut::new(
            Identity::from_inner_ref(context),
            DenseSlicesMut::new(context, entities, metas.as_identity_slice_mut()),
        );

        let inner = EpochSparseViewMut::new(dense, sparse)?.into_mut_view_ptr();
        let me = Self::from_inner(inner);
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        entities: &'a mut [Entity],
        metas: &'a mut [Meta],
        sparse: &'a mut [SparseItem<Entity>],
    ) -> Self {
        let context = Self::CONTEXT;
        let dense = unsafe {
            SoaSlicesMut::new(
                Identity::from_inner_ref(context),
                DenseSlicesMut::new_unchecked(entities, metas.as_identity_slice_mut()),
            )
        };

        let inner = unsafe { EpochSparseViewMut::from_parts(dense, sparse) }.into_mut_view_ptr();
        Self::from_inner(inner)
    }

    #[inline]
    pub(super) fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    #[allow(unused)]
    pub(super) fn into_inner(self) -> Inner<'a, Meta> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub unsafe fn into_parts(
        self,
    ) -> (
        &'a mut [Entity],
        &'a mut [Meta],
        &'a mut [SparseItem<Entity>],
    ) {
        let Self { inner } = self;

        let (dense, sparse) = inner.into_mut_slice_ptrs();
        let sparse = unsafe { sparse.as_mut_unchecked() };

        let context = Self::CONTEXT;
        let (entities, metas) = unsafe { dense.as_mut_unchecked(context).into_parts() };
        let metas = metas.as_inner_mut();

        (entities, metas, sparse)
    }

    #[inline]
    pub fn as_view(&self) -> EntityRegistryView<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.cast_const();
        EntityRegistryView::from_inner(inner)
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> EntityRegistryViewMut<'_, Meta> {
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
    pub fn as_slices(&self) -> (&[Entity], &[Meta], &[SparseItem<Entity>]) {
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
    pub fn as_sparse(&self) -> &[SparseItem<Entity>] {
        let (_, _, sparse) = self.as_slices();
        sparse
    }

    #[inline]
    pub unsafe fn as_mut_slices(
        &mut self,
    ) -> (&mut [Entity], &mut [Meta], &mut [SparseItem<Entity>]) {
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
    pub fn as_ptrs(&self) -> (*const Entity, *const Meta, *const SparseItem<Entity>) {
        let (entities, metas, sparse) = self.as_slices();
        (entities.as_ptr(), metas.as_ptr(), sparse.as_ptr())
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut Entity, *mut Meta, *mut SparseItem<Entity>) {
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

impl<Meta> Debug for EntityRegistryViewMut<'_, Meta>
where
    Meta: Debug,
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

impl<Meta> Default for EntityRegistryViewMut<'_, Meta> {
    fn default() -> Self {
        let inner = Inner::from(Self::CONTEXT);
        Self::from_inner(inner)
    }
}

impl<Meta> PartialEq for EntityRegistryViewMut<'_, Meta>
where
    Meta: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slices() == other.as_slices()
    }
}

impl<Meta> Eq for EntityRegistryViewMut<'_, Meta> where Meta: Eq {}

impl<Meta> PartialOrd for EntityRegistryViewMut<'_, Meta>
where
    Meta: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let slices = self.as_slices();
        let other = other.as_slices();
        slices.partial_cmp(&other)
    }
}

impl<Meta> Ord for EntityRegistryViewMut<'_, Meta>
where
    Meta: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let slices = self.as_slices();
        let other = other.as_slices();
        slices.cmp(&other)
    }
}

impl<Meta> Hash for EntityRegistryViewMut<'_, Meta>
where
    Meta: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slices().hash(state);
    }
}

impl<Meta> AsRef<[Entity]> for EntityRegistryViewMut<'_, Meta> {
    #[inline]
    fn as_ref(&self) -> &[Entity] {
        self.as_entities()
    }
}

impl<Meta> Index<Entity> for EntityRegistryViewMut<'_, Meta> {
    type Output = Meta;

    #[inline]
    fn index(&self, entity: Entity) -> &Self::Output {
        let Self { inner } = self;
        unsafe { inner.as_ref_unchecked() }.into_index(entity)
    }
}

impl<Meta> IndexMut<Entity> for EntityRegistryViewMut<'_, Meta> {
    #[inline]
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        let Self { inner } = self;
        unsafe { inner.as_mut_unchecked() }.into_index_mut(entity)
    }
}

impl<'a, Meta> IntoIterator for &'a EntityRegistryViewMut<'_, Meta> {
    type Item = (Entity, &'a Meta);
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta> IntoIterator for &'a mut EntityRegistryViewMut<'_, Meta> {
    type Item = (Entity, &'a mut Meta);
    type IntoIter = IterMut<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, Meta> IntoIterator for EntityRegistryViewMut<'a, Meta> {
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
impl<'a, Meta> rayon::iter::IntoParallelIterator for &'a EntityRegistryViewMut<'_, Meta>
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
impl<'a, Meta> rayon::iter::IntoParallelIterator for &'a mut EntityRegistryViewMut<'_, Meta>
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

#[cfg(feature = "rayon")]
impl<'a, Meta> rayon::iter::IntoParallelIterator for EntityRegistryViewMut<'a, Meta>
where
    Meta: Send,
{
    type Item = (Entity, &'a mut Meta);
    type Iter = crate::registry::ParIterMut<'a, Meta>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        crate::registry::ParIterMut::new(self)
    }
}

unsafe impl<Meta> Send for EntityRegistryViewMut<'_, Meta> where Meta: Send {}
unsafe impl<Meta> Sync for EntityRegistryViewMut<'_, Meta> where Meta: Sync {}
