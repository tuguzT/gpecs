use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem,
};

use bytemuck::must_cast_slice_mut;
use gpecs_entity::Entity;
use gpecs_sparse::{
    error::FromPartsError,
    item::{DefaultSparseItem, KeyValueMutSlices},
    soa::{
        identity::Identity,
        slice::SoaSlicesMut,
        traits::{Slices, SoaContext},
    },
    view::{EpochSparseView, EpochSparseViewMut},
};

use crate::{
    bundle::{Bundle, BundleRefs, BundleRefsMut, BundleSlices, BundleSlicesMut},
    storage::{BundleIter, BundleIterMut, Bundles, NoEpochEntity},
};

type Inner<'a, B> = EpochSparseViewMut<'static, 'a, NoEpochEntity, B>;

pub struct BundlesMut<'a, B>
where
    B: Bundle,
{
    inner: Inner<'a, B>,
}

impl<'a, B> BundlesMut<'a, B>
where
    B: Bundle,
{
    #[inline]
    pub fn new(
        entities: &'a mut [Entity],
        bundles: BundleSlicesMut<'a, B>,
        sparse: &'a mut [DefaultSparseItem<NoEpochEntity>],
    ) -> Result<Self, FromPartsError<NoEpochEntity>> {
        let entities = must_cast_slice_mut(entities);
        let slices = KeyValueMutSlices::new(B::CONTEXT, entities, bundles);
        let dense = SoaSlicesMut::new(Identity::from_inner_ref(B::CONTEXT), slices);

        let inner = EpochSparseViewMut::new(dense, sparse)?;
        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        entities: &'a mut [Entity],
        bundles: BundleSlicesMut<'a, B>,
        sparse: &'a mut [DefaultSparseItem<NoEpochEntity>],
    ) -> Self {
        let entities = must_cast_slice_mut(entities);
        let slices = KeyValueMutSlices::new(B::CONTEXT, entities, bundles);
        let dense = SoaSlicesMut::new(Identity::from_inner_ref(B::CONTEXT), slices);

        let inner = unsafe { EpochSparseViewMut::from_parts(dense, sparse) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub(super) unsafe fn from_inner(inner: Inner<'a, B>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn into_parts(self) -> Parts<'a, B> {
        let Self { inner } = self;

        let (_, dense, sparse) = unsafe { inner.into_mut_slices_with_context() };
        let (entities, bundles) = dense.into_parts();
        let entities = must_cast_slice_mut(entities);
        (entities, bundles, sparse)
    }

    #[inline]
    pub fn into_entities(self) -> &'a [Entity] {
        let (entities, _, _) = unsafe { self.into_parts() };
        entities
    }

    #[inline]
    pub fn into_bundle_slices(self) -> BundleSlices<'a, B> {
        let bundles = self.into_mut_bundle_slices();
        B::CONTEXT.mut_slices_as_slices(bundles)
    }

    #[inline]
    pub fn into_mut_bundle_slices(self) -> BundleSlicesMut<'a, B> {
        let (_, bundles, _) = unsafe { self.into_parts() };
        bundles
    }

    #[inline]
    pub fn into_sparse(self) -> &'a [DefaultSparseItem<NoEpochEntity>] {
        let (_, _, sparse) = unsafe { self.into_parts() };
        sparse
    }

    #[inline]
    pub fn as_bundles(&self) -> Bundles<'_, B> {
        let Self { inner } = self;

        let inner = unsafe { map_view_context(inner.as_view()) };
        unsafe { Bundles::from_inner(inner) }
    }

    #[inline]
    pub fn as_mut_bundles(&mut self) -> BundlesMut<'_, B> {
        let Self { inner } = self;

        let inner = unsafe { map_mut_view_context(inner.as_mut_view()) };
        unsafe { BundlesMut::from_inner(inner) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { inner, .. } = self;
        inner.is_empty()
    }

    #[inline]
    pub fn sparse_len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.sparse_len()
    }

    #[inline]
    pub fn sparse_is_empty(&self) -> bool {
        let Self { inner, .. } = self;
        inner.sparse_is_empty()
    }

    #[inline]
    pub fn as_slices(&self) -> AsSlices<'_, B> {
        self.as_bundles().into_parts()
    }

    #[inline]
    pub fn as_entities(&self) -> &[Entity] {
        self.as_bundles().into_entities()
    }

    #[inline]
    pub fn as_bundle_slices(&self) -> BundleSlices<'_, B> {
        self.as_bundles().into_bundle_slices()
    }

    #[inline]
    pub fn as_sparse(&self) -> &[DefaultSparseItem<NoEpochEntity>] {
        self.as_bundles().into_sparse()
    }

    #[inline]
    pub unsafe fn as_mut_slices(&mut self) -> Parts<'_, B> {
        unsafe { self.as_mut_bundles().into_parts() }
    }

    #[inline]
    pub fn as_mut_bundle_slices(&mut self) -> BundleSlicesMut<'_, B> {
        self.as_mut_bundles().into_mut_bundle_slices()
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { inner, .. } = self;
        inner.contains_key(entity.into())
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<BundleRefs<'_, B>> {
        self.as_bundles().into_get(entity)
    }

    #[inline]
    pub fn into_get(self, entity: Entity) -> Option<BundleRefs<'a, B>> {
        let Self { inner, .. } = self;
        inner.into_get(entity.into())
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<BundleRefsMut<'_, B>> {
        self.as_mut_bundles().into_get_mut(entity)
    }

    #[inline]
    pub fn into_get_mut(self, entity: Entity) -> Option<BundleRefsMut<'a, B>> {
        let Self { inner, .. } = self;
        inner.into_get_mut(entity.into())
    }

    #[inline]
    pub fn iter(&self) -> BundleIter<'_, B> {
        self.as_bundles().into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> BundleIterMut<'_, B> {
        self.as_mut_bundles().into_iter()
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter(&self) -> crate::storage::BundleParIter<'_, B> {
        self.as_bundles().into_par_iter()
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_mut(&mut self) -> crate::storage::BundleParIterMut<'_, B> {
        self.as_mut_bundles().into_par_iter()
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_iter(self) -> crate::storage::BundleParIterMut<'a, B> {
        let Self { inner } = self;

        let inner = inner.into_par_iter();
        crate::storage::BundleParIterMut::new(inner)
    }
}

impl<B> Debug for BundlesMut<'_, B>
where
    B: Bundle,
    for<'a> BundleSlices<'a, B>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, bundles, sparse) = &self.as_slices();
        f.debug_struct("Bundles")
            .field("entities", entities)
            .field("bundles", bundles)
            .field("sparse", sparse)
            .finish()
    }
}

impl<B> Default for BundlesMut<'_, B>
where
    B: Bundle,
{
    #[inline]
    fn default() -> Self {
        let inner = Inner::from(B::CONTEXT);
        Self { inner }
    }
}

impl<B> PartialEq for BundlesMut<'_, B>
where
    B: Bundle,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner } = self;
        inner == &other.inner
    }
}

impl<B> Eq for BundlesMut<'_, B>
where
    B: Bundle,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Eq,
{
}

impl<B> PartialOrd for BundlesMut<'_, B>
where
    B: Bundle,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner } = self;
        inner.partial_cmp(&other.inner)
    }
}

impl<B> Ord for BundlesMut<'_, B>
where
    B: Bundle,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner } = self;
        inner.cmp(&other.inner)
    }
}

impl<B> Hash for BundlesMut<'_, B>
where
    B: Bundle,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner } = self;
        inner.hash(state);
    }
}

impl<'a, B> IntoIterator for &'a BundlesMut<'_, B>
where
    B: Bundle,
{
    type Item = (Entity, BundleRefs<'a, B>);
    type IntoIter = BundleIter<'a, B>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, B> IntoIterator for &'a mut BundlesMut<'_, B>
where
    B: Bundle,
{
    type Item = (Entity, BundleRefsMut<'a, B>);
    type IntoIter = BundleIterMut<'a, B>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, B> IntoIterator for BundlesMut<'a, B>
where
    B: Bundle,
{
    type Item = (Entity, BundleRefsMut<'a, B>);
    type IntoIter = BundleIterMut<'a, B>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        BundleIterMut::from_inner(inner)
    }
}

#[cfg(feature = "rayon")]
impl<'a, B> rayon::iter::IntoParallelIterator for &'a BundlesMut<'_, B>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Sync,
    BundleRefs<'a, B>: Send,
{
    type Item = (Entity, BundleRefs<'a, B>);
    type Iter = crate::storage::BundleParIter<'a, B>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, B> rayon::iter::IntoParallelIterator for &'a mut BundlesMut<'_, B>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Send,
    BundleRefsMut<'a, B>: Send,
{
    type Item = (Entity, BundleRefsMut<'a, B>);
    type Iter = crate::storage::BundleParIterMut<'a, B>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter_mut()
    }
}

#[cfg(feature = "rayon")]
impl<'a, B> rayon::iter::IntoParallelIterator for BundlesMut<'a, B>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Send,
    BundleRefsMut<'a, B>: Send,
{
    type Item = (Entity, BundleRefsMut<'a, B>);
    type Iter = crate::storage::BundleParIterMut<'a, B>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.into_par_iter()
    }
}

type Parts<'a, B> = (
    &'a mut [Entity],
    BundleSlicesMut<'a, B>,
    &'a mut [DefaultSparseItem<NoEpochEntity>],
);

type AsSlices<'a, B> = (
    &'a [Entity],
    BundleSlices<'a, B>,
    &'a [DefaultSparseItem<NoEpochEntity>],
);

#[inline]
unsafe fn map_view_context<'a, B>(
    view: EpochSparseView<'_, 'a, NoEpochEntity, B>,
) -> EpochSparseView<'static, 'a, NoEpochEntity, B>
where
    B: Bundle,
{
    unsafe { mem::transmute(view) }
}

#[inline]
unsafe fn map_mut_view_context<'a, B>(
    view: EpochSparseViewMut<'_, 'a, NoEpochEntity, B>,
) -> Inner<'a, B>
where
    B: Bundle,
{
    unsafe { mem::transmute(view) }
}
