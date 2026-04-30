use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::Index,
};

use gpecs_sparse::{
    error::FromPartsError,
    item::{DenseSlices, SparseItem},
    soa::{
        identity::{AsIdentitySlice, Identity, IdentitySlice},
        slice::SoaSlices,
    },
    view::{EpochSparseView, EpochSparseViewPtr},
};

use crate::{
    entity::{Entity, EntityEpoch},
    registry::Iter,
};

type Inner<'a, Meta> = EpochSparseViewPtr<'a, Entity, Identity<Meta>>;

#[repr(transparent)]
pub struct EntityRegistryView<'a, Meta>
where
    Meta: 'a,
{
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> EntityRegistryView<'a, Meta> {
    const CONTEXT: &'a () = &();

    #[inline]
    pub fn new(
        entities: &'a [Entity],
        metas: &'a [Meta],
        sparse: &'a [SparseItem<Entity>],
    ) -> Result<Self, FromPartsError<Entity>> {
        let context = Self::CONTEXT;
        let dense = SoaSlices::new(
            Identity::from_inner_ref(context),
            DenseSlices::new(context, entities, metas.as_identity_slice()),
        );

        let inner = EpochSparseView::new(dense, sparse)?.into_view_ptr();
        let me = Self::from_inner(inner);
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        entities: &'a [Entity],
        metas: &'a [Meta],
        sparse: &'a [SparseItem<Entity>],
    ) -> Self {
        let context = Self::CONTEXT;
        let dense = unsafe {
            SoaSlices::new(
                Identity::from_inner_ref(context),
                DenseSlices::new_unchecked(entities, metas.as_identity_slice()),
            )
        };

        let inner = unsafe { EpochSparseView::from_parts(dense, sparse) }.into_view_ptr();
        Self::from_inner(inner)
    }

    #[inline]
    pub(super) fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a [Entity], &'a [Meta], &'a [SparseItem<Entity>]) {
        let Self { inner } = self;

        let (dense, sparse) = inner.into_slice_ptrs();
        let sparse = unsafe { sparse.as_ref_unchecked() };

        let context = Self::CONTEXT;
        let (entities, metas) = unsafe { dense.as_ref_unchecked(context).into_parts() };
        let metas = metas.as_inner();

        (entities, metas, sparse)
    }

    #[inline]
    pub fn as_view(&self) -> EntityRegistryView<'_, Meta> {
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
    pub fn as_ptrs(&self) -> (*const Entity, *const Meta, *const SparseItem<Entity>) {
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
}

impl<Meta> Debug for EntityRegistryView<'_, Meta>
where
    Meta: Debug,
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

impl<Meta> Default for EntityRegistryView<'_, Meta> {
    fn default() -> Self {
        let inner = Inner::from(Self::CONTEXT);
        Self::from_inner(inner)
    }
}

impl<Meta> Clone for EntityRegistryView<'_, Meta> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta> Copy for EntityRegistryView<'_, Meta> {}

impl<Meta> PartialEq for EntityRegistryView<'_, Meta>
where
    Meta: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slices() == other.as_slices()
    }
}

impl<Meta> Eq for EntityRegistryView<'_, Meta> where Meta: Eq {}

impl<Meta> PartialOrd for EntityRegistryView<'_, Meta>
where
    Meta: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let slices = self.as_slices();
        let other = other.as_slices();
        slices.partial_cmp(&other)
    }
}

impl<Meta> Ord for EntityRegistryView<'_, Meta>
where
    Meta: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let slices = self.as_slices();
        let other = other.as_slices();
        slices.cmp(&other)
    }
}

impl<Meta> Hash for EntityRegistryView<'_, Meta>
where
    Meta: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slices().hash(state);
    }
}

impl<Meta> AsRef<[Entity]> for EntityRegistryView<'_, Meta> {
    #[inline]
    fn as_ref(&self) -> &[Entity] {
        self.as_entities()
    }
}

impl<Meta> Index<Entity> for EntityRegistryView<'_, Meta> {
    type Output = Meta;

    #[inline]
    fn index(&self, entity: Entity) -> &Self::Output {
        self.into_index(entity)
    }
}

impl<'a, Meta> IntoIterator for &'a EntityRegistryView<'_, Meta> {
    type Item = (Entity, &'a Meta);
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta> IntoIterator for EntityRegistryView<'a, Meta> {
    type Item = (Entity, &'a Meta);
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        Iter::from_inner(inner)
    }
}

unsafe impl<Meta> Send for EntityRegistryView<'_, Meta> where Meta: Sync {}
unsafe impl<Meta> Sync for EntityRegistryView<'_, Meta> where Meta: Sync {}
