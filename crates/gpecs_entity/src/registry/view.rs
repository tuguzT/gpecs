use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::Index,
};

use gpecs_sparse::{
    error::FromPartsError,
    item::{DefaultSparseItem, KeyValueSlices, SparseItem},
    soa::{
        identity::{AsIdentitySlice, Identity, IdentitySlice},
        slice::SoaSlices,
    },
    view::{EpochSparseView, EpochSparseViewPtr},
};

use crate::{Entity, EntityEpoch, registry::Iter};

type Inner<'a, Meta, S> = EpochSparseViewPtr<'a, Entity, Identity<Meta>, S>;

#[repr(transparent)]
pub struct EntityRegistryView<'a, Meta, S = DefaultSparseItem<Entity>>
where
    Meta: 'a,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + 'a,
{
    inner: Inner<'a, Meta, S>,
}

impl<'a, Meta, S> EntityRegistryView<'a, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    const CONTEXT: &'a () = &();

    #[inline]
    pub fn new(
        entities: &'a [Entity],
        metas: &'a [Meta],
        sparse: &'a [S],
    ) -> Result<Self, FromPartsError<Entity>> {
        let context = Self::CONTEXT;
        let dense = SoaSlices::new(
            Identity::from_inner_ref(context),
            KeyValueSlices::new(context, entities, metas.as_identity_slice()),
        );

        let inner = EpochSparseView::new(dense, sparse)?.into_view_ptr();
        let me = Self::from_inner(inner);
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(entities: &'a [Entity], metas: &'a [Meta], sparse: &'a [S]) -> Self {
        let context = Self::CONTEXT;
        let dense = unsafe {
            SoaSlices::new(
                Identity::from_inner_ref(context),
                KeyValueSlices::new_unchecked(entities, metas.as_identity_slice()),
            )
        };

        let inner = unsafe { EpochSparseView::from_parts(dense, sparse) }.into_view_ptr();
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
    pub fn into_parts(self) -> (&'a [Entity], &'a [Meta], &'a [S]) {
        let Self { inner } = self;

        let (dense, sparse) = inner.into_slice_ptrs();
        let sparse = unsafe { sparse.as_ref_unchecked() };

        let context = Self::CONTEXT;
        let (entities, metas) = unsafe { dense.as_ref_unchecked(context).into_parts() };
        let metas = metas.as_inner();

        (entities, metas, sparse)
    }

    #[inline]
    pub fn as_view(&self) -> EntityRegistryView<'_, Meta, S> {
        let Self { inner } = *self;
        EntityRegistryView::from_inner(inner)
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
    pub fn as_ptrs(&self) -> (*const Entity, *const Meta, *const S) {
        let (entities, metas, sparse) = self.as_slices();
        (entities.as_ptr(), metas.as_ptr(), sparse.as_ptr())
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { inner } = self;
        unsafe { inner.as_ref_unchecked() }.contains_key(entity)
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&Meta> {
        self.into_get(entity)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<EntityEpoch> {
        let Self { inner } = self;

        let sparse_index = sparse_index.try_into().ok()?;
        unsafe { inner.as_ref_unchecked() }.get_epoch(sparse_index)
    }

    #[inline]
    pub fn into_get(self, entity: Entity) -> Option<&'a Meta> {
        let Self { inner } = self;

        unsafe { inner.as_ref_unchecked() }
            .into_get(entity)
            .map(Identity::as_inner)
    }

    #[inline]
    pub fn into_index(self, entity: Entity) -> &'a Meta {
        let Self { inner } = self;
        unsafe { inner.as_ref_unchecked() }.into_index(entity)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Meta> {
        (*self).into_iter()
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter(&self) -> crate::registry::ParIter<'_, Meta, S> {
        crate::registry::ParIter::new(*self)
    }
}

impl<Meta, S> Debug for EntityRegistryView<'_, Meta, S>
where
    Meta: Debug,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, metas, sparse) = self.as_slices();
        f.debug_struct("EntityRegistryView")
            .field("entities", &entities)
            .field("metas", &metas)
            .field("sparse", &sparse)
            .finish()
    }
}

impl<Meta, S> Default for EntityRegistryView<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    fn default() -> Self {
        let inner = Inner::from(Self::CONTEXT);
        Self::from_inner(inner)
    }
}

impl<Meta, S> Clone for EntityRegistryView<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta, S> Copy for EntityRegistryView<'_, Meta, S> where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>
{
}

impl<Meta, S> PartialEq for EntityRegistryView<'_, Meta, S>
where
    Meta: PartialEq,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slices() == other.as_slices()
    }
}

impl<Meta, S> Eq for EntityRegistryView<'_, Meta, S>
where
    Meta: Eq,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Eq,
{
}

impl<Meta, S> PartialOrd for EntityRegistryView<'_, Meta, S>
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

impl<Meta, S> Ord for EntityRegistryView<'_, Meta, S>
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

impl<Meta, S> Hash for EntityRegistryView<'_, Meta, S>
where
    Meta: Hash,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slices().hash(state);
    }
}

impl<Meta, S> AsRef<[Entity]> for EntityRegistryView<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    #[inline]
    fn as_ref(&self) -> &[Entity] {
        self.as_entities()
    }
}

impl<Meta, S> Index<Entity> for EntityRegistryView<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    type Output = Meta;

    #[inline]
    fn index(&self, entity: Entity) -> &Self::Output {
        self.into_index(entity)
    }
}

impl<'a, Meta, S> IntoIterator for &'a EntityRegistryView<'_, Meta, S>
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

impl<'a, Meta, S> IntoIterator for EntityRegistryView<'a, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    type Item = (Entity, &'a Meta);
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        Iter::from_inner(inner)
    }
}

#[cfg(feature = "rayon")]
impl<'a, Meta, S> rayon::iter::IntoParallelIterator for &'a EntityRegistryView<'_, Meta, S>
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
impl<'a, Meta, S> rayon::iter::IntoParallelIterator for EntityRegistryView<'a, Meta, S>
where
    Meta: Sync,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Sync,
{
    type Item = (Entity, &'a Meta);
    type Iter = crate::registry::ParIter<'a, Meta, S>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        crate::registry::ParIter::new(self)
    }
}

unsafe impl<Meta, S> Send for EntityRegistryView<'_, Meta, S>
where
    Meta: Sync,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Sync,
{
}

unsafe impl<Meta, S> Sync for EntityRegistryView<'_, Meta, S>
where
    Meta: Sync,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Sync,
{
}
