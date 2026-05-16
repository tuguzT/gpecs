use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem,
};

use bytemuck::must_cast_slice;
use gpecs_entity::{Entity, NoEpochEntity, NoEpochEntitySparseItem};
use gpecs_sparse::{
    error::FromPartsError,
    item::{KeyValueSlices, SparseItem},
    soa::{
        identity::Identity,
        slice::SoaSlices,
        traits::{Ptrs, Slices},
    },
    view::EpochSparseView,
};

use crate::{
    bundle::{Bundle, BundleRefs, BundleSlices},
    storage::BundleIter,
};

type Inner<'a, B, S> = EpochSparseView<'static, 'a, NoEpochEntity, B, S>;

pub struct Bundles<'a, B, S = NoEpochEntitySparseItem>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()> + 'a,
{
    inner: Inner<'a, B, S>,
}

impl<'a, B, S> Bundles<'a, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    #[inline]
    pub fn new(
        entities: &'a [Entity],
        bundles: BundleSlices<'a, B>,
        sparse: &'a [S],
    ) -> Result<Self, FromPartsError<NoEpochEntity>> {
        let entities = must_cast_slice(entities);
        let slices = KeyValueSlices::new(B::CONTEXT, entities, bundles);
        let dense = SoaSlices::new(Identity::from_inner_ref(B::CONTEXT), slices);

        let inner = EpochSparseView::new(dense, sparse)?;
        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        entities: &'a [Entity],
        bundles: BundleSlices<'a, B>,
        sparse: &'a [S],
    ) -> Self {
        let entities = must_cast_slice(entities);
        let slices = KeyValueSlices::new(B::CONTEXT, entities, bundles);
        let dense = SoaSlices::new(Identity::from_inner_ref(B::CONTEXT), slices);

        let inner = unsafe { EpochSparseView::from_parts(dense, sparse) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub(super) unsafe fn from_inner(inner: Inner<'a, B, S>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_parts(self) -> Parts<'a, B, S> {
        let Self { inner } = self;

        let (_, dense, sparse) = inner.into_slices_with_context();
        let (entities, bundles) = dense.into_parts();
        let entities = must_cast_slice(entities);
        (entities, bundles, sparse)
    }

    #[inline]
    pub fn into_entities(self) -> &'a [Entity] {
        let (entities, _, _) = self.into_parts();
        entities
    }

    #[inline]
    pub fn into_bundle_slices(self) -> BundleSlices<'a, B> {
        let (_, bundles, _) = self.into_parts();
        bundles
    }

    #[inline]
    pub fn into_sparse(self) -> &'a [S] {
        let (_, _, sparse) = self.into_parts();
        sparse
    }

    #[inline]
    pub fn as_bundles(&self) -> Bundles<'_, B, S> {
        let Self { inner } = self;

        let inner = unsafe { map_view_context(inner.as_view()) };
        unsafe { Bundles::from_inner(inner) }
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
    pub fn as_slices(&self) -> Parts<'_, B, S> {
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
    pub fn iter(&self) -> BundleIter<'_, B> {
        self.as_bundles().into_iter()
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter(&self) -> crate::storage::BundleParIter<'_, B> {
        self.as_bundles().into_par_iter()
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_iter(self) -> crate::storage::BundleParIter<'a, B> {
        let Self { inner } = self;

        let inner = inner.into_par_iter();
        crate::storage::BundleParIter::new(inner)
    }
}

impl<B, S> Debug for Bundles<'_, B, S>
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

impl<B, S> Default for Bundles<'_, B, S>
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

impl<B, S> Clone for Bundles<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<B, S> Copy for Bundles<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
    for<'ctx> Ptrs<'ctx, B>: Copy,
{
}

impl<B, S> PartialEq for Bundles<'_, B, S>
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

impl<B, S> Eq for Bundles<'_, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()> + Eq,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Eq,
{
}

impl<B, S> PartialOrd for Bundles<'_, B, S>
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

impl<B, S> Ord for Bundles<'_, B, S>
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

impl<B, S> Hash for Bundles<'_, B, S>
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

impl<'a, B, S> IntoIterator for &'a Bundles<'_, B, S>
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

impl<'a, B, S> IntoIterator for Bundles<'a, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    type Item = (Entity, BundleRefs<'a, B>);
    type IntoIter = BundleIter<'a, B>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        BundleIter::from_inner(inner)
    }
}

#[cfg(feature = "rayon")]
impl<'a, B, S> rayon::iter::IntoParallelIterator for &'a Bundles<'_, B, S>
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
impl<'a, B, S> rayon::iter::IntoParallelIterator for Bundles<'a, B, S>
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
        self.into_par_iter()
    }
}

type Parts<'a, B, S> = (&'a [Entity], BundleSlices<'a, B>, &'a [S]);

#[inline]
unsafe fn map_view_context<'a, B, S>(
    view: EpochSparseView<'_, 'a, NoEpochEntity, B, S>,
) -> Inner<'a, B, S>
where
    B: Bundle,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    unsafe { mem::transmute(view) }
}
