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
    item::{DefaultSparseItem, KeyValueMutSlices, SparseItem},
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

type Inner<'a, B, S> = EpochSparseViewMut<'static, 'a, NoEpochEntity, B, S>;

pub struct BundlesMut<'a, B, S = DefaultSparseItem<NoEpochEntity>>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()> + 'a,
{
    inner: Inner<'a, B, S>,
}

impl<'a, B, S> BundlesMut<'a, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    #[inline]
    pub fn new(
        entities: &'a mut [Entity],
        bundles: BundleSlicesMut<'a, B>,
        sparse: &'a mut [S],
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
        sparse: &'a mut [S],
    ) -> Self {
        let entities = must_cast_slice_mut(entities);
        let slices = KeyValueMutSlices::new(B::CONTEXT, entities, bundles);
        let dense = SoaSlicesMut::new(Identity::from_inner_ref(B::CONTEXT), slices);

        let inner = unsafe { EpochSparseViewMut::from_parts(dense, sparse) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub(super) unsafe fn from_inner(inner: Inner<'a, B, S>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn into_parts(self) -> Parts<'a, B, S> {
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
    pub fn into_sparse(self) -> &'a [S] {
        let (_, _, sparse) = unsafe { self.into_parts() };
        sparse
    }

    #[inline]
    pub fn as_bundles(&self) -> Bundles<'_, B, S> {
        let Self { inner } = self;

        let inner = unsafe { map_view_context(inner.as_view()) };
        unsafe { Bundles::from_inner(inner) }
    }

    #[inline]
    pub fn as_mut_bundles(&mut self) -> BundlesMut<'_, B, S> {
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
    pub fn as_slices(&self) -> AsSlices<'_, B, S> {
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
    pub fn as_sparse(&self) -> &[S] {
        self.as_bundles().into_sparse()
    }

    #[inline]
    pub unsafe fn as_mut_slices(&mut self) -> Parts<'_, B, S> {
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

impl<B, S> Debug for BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()> + Debug,
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

impl<B, S> Default for BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    #[inline]
    fn default() -> Self {
        let inner = Inner::from(B::CONTEXT);
        Self { inner }
    }
}

impl<B, S> PartialEq for BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()> + PartialEq,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner } = self;
        inner == &other.inner
    }
}

impl<B, S> Eq for BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()> + Eq,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Eq,
{
}

impl<B, S> PartialOrd for BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()> + PartialOrd,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner } = self;
        inner.partial_cmp(&other.inner)
    }
}

impl<B, S> Ord for BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()> + Ord,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner } = self;
        inner.cmp(&other.inner)
    }
}

impl<B, S> Hash for BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()> + Hash,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner } = self;
        inner.hash(state);
    }
}

impl<'a, B, S> IntoIterator for &'a BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    type Item = (Entity, BundleRefs<'a, B>);
    type IntoIter = BundleIter<'a, B>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, B, S> IntoIterator for &'a mut BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    type Item = (Entity, BundleRefsMut<'a, B>);
    type IntoIter = BundleIterMut<'a, B>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, B, S> IntoIterator for BundlesMut<'a, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
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
impl<'a, B, S> rayon::iter::IntoParallelIterator for &'a BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
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
impl<'a, B, S> rayon::iter::IntoParallelIterator for &'a mut BundlesMut<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
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
impl<'a, B, S> rayon::iter::IntoParallelIterator for BundlesMut<'a, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
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

type Parts<'a, B, S> = (&'a mut [Entity], BundleSlicesMut<'a, B>, &'a mut [S]);
type AsSlices<'a, B, S> = (&'a [Entity], BundleSlices<'a, B>, &'a [S]);

#[inline]
unsafe fn map_view_context<'a, B, S>(
    view: EpochSparseView<'_, 'a, NoEpochEntity, B, S>,
) -> EpochSparseView<'static, 'a, NoEpochEntity, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    unsafe { mem::transmute(view) }
}

#[inline]
unsafe fn map_mut_view_context<'a, B, S>(
    view: EpochSparseViewMut<'_, 'a, NoEpochEntity, B, S>,
) -> Inner<'a, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    unsafe { mem::transmute(view) }
}
