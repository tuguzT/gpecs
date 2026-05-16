use core::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use bytemuck::must_cast_slice_mut;
use gpecs_component::registry::{
    ComponentRegistryView,
    traits::{ComponentIdFrom, FromComponentType},
};
use gpecs_entity::{Entity, NoEpochEntity};
use gpecs_soa_erased::{
    ptr::slice::PtrsItem,
    soa::{
        field::FieldLayouts,
        identity::Identity,
        slice::SoaSlicesMut,
        traits::{
            Refs as ErasedBundleRefs, RefsMut as ErasedBundleRefsMut, Slices as ErasedBundles,
            SlicesMut as ErasedBundlesMut,
        },
    },
};
use gpecs_sparse::{
    error::FromPartsError,
    item::{DefaultSparseItem, KeyValueMutSlices, SparseItem},
    view::{EpochSparseViewMut, EpochSparseViewMutPtr},
};

use crate::{
    bundle::{Bundle, BundleRefs, BundleRefsMut, erased::error::DowncastError},
    erased::{ErasedArchetypeView, error::IncompatibleArchetypeError},
    storage::{
        ArchetypeStorageView, BundleIter, BundleIterMut, Bundles, BundlesMut, Iter, IterMut,
        traits::ErasedArchetypeSoa,
    },
};

type Inner<'ctx, T, S> = EpochSparseViewMutPtr<'ctx, NoEpochEntity, T, S>;

#[repr(transparent)]
pub struct ArchetypeStorageViewMut<'ctx, 'a, T, S = DefaultSparseItem<NoEpochEntity>>
where
    T: ErasedArchetypeSoa + ?Sized,
    S: SparseItem<Index = u32, Epoch = ()> + 'a,
{
    inner: Inner<'ctx, T, S>,
    phantom: PhantomData<&'a mut [PtrsItem<T::Ptrs>]>,
}

impl<'ctx, 'a, T, S> ArchetypeStorageViewMut<'ctx, 'a, T, S>
where
    T: ErasedArchetypeSoa + ?Sized,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    #[inline]
    pub fn new(
        context: &'ctx T::Context,
        entities: &'a mut [Entity],
        bundles: ErasedBundlesMut<'ctx, 'a, T>,
        sparse: &'a mut [S],
    ) -> Result<Self, FromPartsError<NoEpochEntity>> {
        let entities = must_cast_slice_mut(entities);
        let dense = SoaSlicesMut::new(
            Identity::from_inner_ref(context),
            KeyValueMutSlices::new(context, entities, bundles),
        );

        let inner = EpochSparseViewMut::new(dense, sparse)?.into_mut_view_ptr();
        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        context: &'ctx T::Context,
        entities: &'a mut [Entity],
        bundles: ErasedBundlesMut<'ctx, 'a, T>,
        sparse: &'a mut [S],
    ) -> Self {
        let entities = must_cast_slice_mut(entities);
        let dense = SoaSlicesMut::new(
            Identity::from_inner_ref(context),
            KeyValueMutSlices::new(context, entities, bundles),
        );

        let inner = unsafe { EpochSparseViewMut::from_parts(dense, sparse) }.into_mut_view_ptr();
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub(crate) unsafe fn from_inner(inner: Inner<'ctx, T, S>) -> Self {
        let phantom = PhantomData;
        Self { inner, phantom }
    }

    #[inline]
    pub unsafe fn into_parts(self) -> MutSlicesWithArchetype<'ctx, 'a, T, S> {
        let Self { inner, .. } = self;

        let (context, dense, sparse) = inner.into_mut_slice_ptrs_with_context();
        let archetype = (**context).field_layouts();
        let sparse = unsafe { sparse.as_mut_unchecked() };

        let (entities, bundles) = unsafe { dense.as_mut_unchecked(context) }.into_parts();
        let entities = must_cast_slice_mut(entities);

        (entities, bundles, sparse, archetype)
    }

    #[inline]
    pub fn into_slices(self) -> Slices<'ctx, 'a, T, S> {
        let (entities, bundles, sparse, _) = unsafe { self.into_parts() };
        (entities, bundles.into(), sparse)
    }

    #[inline]
    pub unsafe fn into_mut_slices(self) -> MutSlices<'ctx, 'a, T, S> {
        let (entities, bundles, sparse, _) = unsafe { self.into_parts() };
        (entities, bundles, sparse)
    }

    #[inline]
    pub fn into_entities(self) -> &'a [Entity] {
        let (entities, _, _) = self.into_slices();
        entities
    }

    #[inline]
    pub fn into_erased_bundles(self) -> ErasedBundles<'ctx, 'a, T> {
        let (_, bundles, _) = self.into_slices();
        bundles
    }

    #[inline]
    pub fn into_mut_erased_bundles(self) -> ErasedBundlesMut<'ctx, 'a, T> {
        let (_, bundles, _) = unsafe { self.into_mut_slices() };
        bundles
    }

    #[inline]
    pub fn into_sparse(self) -> &'a [S] {
        let (_, _, sparse) = self.into_slices();
        sparse
    }

    #[inline]
    pub fn as_view(&self) -> ArchetypeStorageView<'_, '_, T, S> {
        let Self { inner, .. } = self;

        let inner = inner.clone().cast_const();
        unsafe { ArchetypeStorageView::from_inner(inner) }
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> ArchetypeStorageViewMut<'_, '_, T, S> {
        let Self { inner, .. } = self;

        let inner = inner.clone();
        unsafe { ArchetypeStorageViewMut::from_inner(inner) }
    }

    #[inline]
    pub fn into_view(self) -> ArchetypeStorageView<'ctx, 'a, T, S> {
        let Self { inner, .. } = self;

        let inner = inner.cast_const();
        unsafe { ArchetypeStorageView::from_inner(inner) }
    }

    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, T::Meta> {
        let Self { inner, .. } = self;
        (**inner.context()).field_layouts()
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
    pub fn as_slices_with_archetype(&self) -> SlicesWithArchetype<'_, '_, T, S> {
        self.as_view().into_parts()
    }

    #[inline]
    pub fn as_slices(&self) -> Slices<'_, '_, T, S> {
        self.as_view().into_slices()
    }

    #[inline]
    pub fn as_entities(&self) -> &[Entity] {
        self.as_view().into_entities()
    }

    #[inline]
    pub fn as_erased_bundles(&self) -> ErasedBundles<'_, '_, T> {
        self.as_view().into_erased_bundles()
    }

    #[inline]
    pub fn as_sparse(&self) -> &[S] {
        self.as_view().into_sparse()
    }

    #[inline]
    pub unsafe fn as_mut_slices_with_archetype(&mut self) -> MutSlicesWithArchetype<'_, '_, T, S> {
        unsafe { self.as_mut_view().into_parts() }
    }

    #[inline]
    pub unsafe fn as_mut_slices(&mut self) -> MutSlices<'_, '_, T, S> {
        unsafe { self.as_mut_view().into_mut_slices() }
    }

    #[inline]
    pub fn as_mut_erased_bundles(&mut self) -> ErasedBundlesMut<'_, '_, T> {
        self.as_mut_view().into_mut_erased_bundles()
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.as_view().contains(entity)
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<ErasedBundleRefs<'_, '_, T>> {
        self.as_view().into_get(entity)
    }

    #[inline]
    pub fn into_get(self, entity: Entity) -> Option<ErasedBundleRefs<'ctx, 'a, T>> {
        self.into_view().into_get(entity)
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<ErasedBundleRefsMut<'_, '_, T>> {
        self.as_mut_view().into_get_mut(entity)
    }

    #[inline]
    pub fn into_get_mut(self, entity: Entity) -> Option<ErasedBundleRefsMut<'ctx, 'a, T>> {
        let Self { inner, .. } = self;
        unsafe { inner.as_mut_unchecked() }.into_get_mut(entity.into())
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, '_, T> {
        let Self { inner, .. } = self;

        let inner = inner.iter();
        Iter::from_inner(inner)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, '_, T> {
        let Self { inner, .. } = self;

        let inner = inner.iter_mut();
        IterMut::from_inner(inner)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn as_bundles_with_archetype<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<(Bundles<'_, B, S>, ErasedArchetypeView<'_, T::Meta>), IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_view().into_bundles_with_archetype::<B>(components)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn into_bundles_with_archetype<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<(Bundles<'a, B, S>, ErasedArchetypeView<'ctx, T::Meta>), IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.into_view()
            .into_bundles_with_archetype::<B>(components)
    }

    #[inline]
    pub fn as_bundles<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<Bundles<'_, B, S>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_view().into_bundles::<B>(components)
    }

    #[inline]
    pub fn into_bundles<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<Bundles<'a, B, S>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.into_view().into_bundles::<B>(components)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn as_mut_bundles_with_archetype<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<(BundlesMut<'_, B, S>, ErasedArchetypeView<'_, T::Meta>), IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_mut_view()
            .into_mut_bundles_with_archetype::<B>(components)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn into_mut_bundles_with_archetype<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<
        (BundlesMut<'a, B, S>, ErasedArchetypeView<'ctx, T::Meta>),
        IncompatibleArchetypeError,
    >
    where
        B: Bundle,
    {
        let (entities, bundles, sparse, archetype) = unsafe { self.into_parts() };
        archetype.check_compatibility_of::<B>(components)?;

        let bundles = bundles
            .downcast::<B>(components)
            .map_err(DowncastError::into_source)
            .expect("archetype compatibility should have been already checked");
        let bundles = unsafe { BundlesMut::from_parts(entities, bundles, sparse) };

        Ok((bundles, archetype))
    }

    #[inline]
    pub fn as_mut_bundles<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundlesMut<'_, B, S>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_mut_view().into_mut_bundles::<B>(components)
    }

    #[inline]
    pub fn into_mut_bundles<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundlesMut<'a, B, S>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let (bundles, _) = self.into_mut_bundles_with_archetype::<B>(components)?;
        Ok(bundles)
    }

    #[inline]
    pub fn get_bundle<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_view().into_get_bundle::<B>(components, entity)
    }

    #[inline]
    pub fn into_get_bundle<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'a, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.into_view().into_get_bundle::<B>(components, entity)
    }

    #[inline]
    pub fn get_bundle_mut<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        entity: Entity,
    ) -> Result<Option<BundleRefsMut<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_mut_view()
            .into_get_bundle_mut::<B>(components, entity)
    }

    #[inline]
    pub fn into_get_bundle_mut<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
        entity: Entity,
    ) -> Result<Option<BundleRefsMut<'a, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.archetype().check_compatibility_of::<B>(components)?;

        let Some(bundle) = self.into_get_mut(entity) else {
            return Ok(None);
        };
        let bundle = bundle
            .downcast::<B>(components)
            .map_err(DowncastError::into_source)
            .expect("archetype compatibility should have been already checked");
        Ok(Some(bundle))
    }

    #[inline]
    pub fn bundle_iter<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleIter<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_view().into_bundle_iter::<B>(components)
    }

    #[inline]
    pub fn bundle_iter_mut<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleIterMut<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_mut_view().into_bundle_iter::<B>(components)
    }

    #[inline]
    pub fn into_bundle_iter<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleIterMut<'a, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let bundles = self.into_mut_bundles::<B>(components)?;
        let iter = bundles.into_iter();
        Ok(iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn bundle_par_iter<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<crate::storage::BundleParIter<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_view().into_bundle_par_iter(components)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn bundle_par_iter_mut<B>(
        &mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<crate::storage::BundleParIterMut<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.as_mut_view().into_bundle_par_iter(components)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_bundle_par_iter<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<crate::storage::BundleParIterMut<'a, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let bundles = self.into_mut_bundles::<B>(components)?;
        let iter = bundles.into_par_iter();
        Ok(iter)
    }
}

impl<T, S> Debug for ArchetypeStorageViewMut<'_, '_, T, S>
where
    T: ErasedArchetypeSoa + ?Sized,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let component_ids = &self.archetype().into_component_ids();
        f.debug_struct("ArchetypeStorageViewMut")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}

impl<'a, T, S> IntoIterator for &'a ArchetypeStorageViewMut<'_, '_, T, S>
where
    T: ErasedArchetypeSoa + ?Sized,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    type Item = (Entity, ErasedBundleRefs<'a, 'a, T>);
    type IntoIter = Iter<'a, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, S> IntoIterator for &'a mut ArchetypeStorageViewMut<'_, '_, T, S>
where
    T: ErasedArchetypeSoa + ?Sized,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    type Item = (Entity, ErasedBundleRefsMut<'a, 'a, T>);
    type IntoIter = IterMut<'a, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'ctx, 'a, T, S> IntoIterator for ArchetypeStorageViewMut<'ctx, 'a, T, S>
where
    T: ErasedArchetypeSoa + ?Sized,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    type Item = (Entity, ErasedBundleRefsMut<'ctx, 'a, T>);
    type IntoIter = IterMut<'ctx, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner, .. } = self;

        let inner = inner.into_iter();
        IterMut::from_inner(inner)
    }
}

impl<'ctx, 'a, T, S> From<ArchetypeStorageViewMut<'ctx, 'a, T, S>>
    for ArchetypeStorageView<'ctx, 'a, T, S>
where
    T: ErasedArchetypeSoa + ?Sized,
    S: SparseItem<Index = u32, Epoch = ()>,
{
    #[inline]
    fn from(view: ArchetypeStorageViewMut<'ctx, 'a, T, S>) -> Self {
        view.into_view()
    }
}

type SlicesWithArchetype<'ctx, 'a, T, S> = (
    &'a [Entity],
    ErasedBundles<'ctx, 'a, T>,
    &'a [S],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type Slices<'ctx, 'a, T, S> = (&'a [Entity], ErasedBundles<'ctx, 'a, T>, &'a [S]);

type MutSlicesWithArchetype<'ctx, 'a, T, S> = (
    &'a mut [Entity],
    ErasedBundlesMut<'ctx, 'a, T>,
    &'a mut [S],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type MutSlices<'ctx, 'a, T, S> = (&'a mut [Entity], ErasedBundlesMut<'ctx, 'a, T>, &'a mut [S]);
