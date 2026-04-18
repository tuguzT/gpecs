use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use bytemuck::must_cast_slice;
use gpecs_soa_erased::ptr::slice::PtrsItem;
use gpecs_sparse::{
    error::FromPartsError,
    item::{DenseSlices, SparseItem},
    view::{EpochSparseView, EpochSparseViewPtr},
};

use crate::{
    archetype::{
        erased::{ErasedArchetypeView, error::IncompatibleArchetypeError},
        storage::{NoEpochEntity, traits::ErasedArchetypeSoa},
    },
    bundle::{Bundle, BundleRefs, BundleSlices},
    component::registry::{
        ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
    entity::Entity,
    soa::{
        field::FieldDescriptors,
        identity::Identity,
        slice::SoaSlices,
        traits::{Refs as ErasedBundleRefs, Slices as ErasedBundles},
    },
};

type Inner<'a, T> = EpochSparseViewPtr<'a, NoEpochEntity, T>;

#[repr(transparent)]
pub struct ArchetypeStorageView<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    inner: Inner<'ctx, T>,
    phantom: PhantomData<&'a [PtrsItem<T::Ptrs>]>,
}

impl<'ctx, 'a, T> ArchetypeStorageView<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    pub fn new(
        context: &'ctx T::Context,
        entities: &'a [Entity],
        bundles: ErasedBundles<'ctx, 'a, T>,
        sparse: &'a [SparseItem<NoEpochEntity>],
    ) -> Result<Self, FromPartsError<NoEpochEntity>> {
        let entities = must_cast_slice(entities);
        let dense = SoaSlices::new(
            Identity::from_inner_ref(context),
            DenseSlices::new(context, entities, bundles),
        );

        let inner = EpochSparseView::new(dense, sparse)?.into_view_ptr();
        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        context: &'ctx T::Context,
        entities: &'a [Entity],
        bundles: ErasedBundles<'ctx, 'a, T>,
        sparse: &'a [SparseItem<NoEpochEntity>],
    ) -> Self {
        let entities = must_cast_slice(entities);
        let dense = SoaSlices::new(
            Identity::from_inner_ref(context),
            DenseSlices::new(context, entities, bundles),
        );

        let inner = unsafe { EpochSparseView::from_parts(dense, sparse) }.into_view_ptr();
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub(super) unsafe fn from_inner(inner: Inner<'ctx, T>) -> Self {
        let phantom = PhantomData;
        Self { inner, phantom }
    }

    #[inline]
    pub fn into_parts(self) -> SlicesWithArchetype<'ctx, 'a, T> {
        let Self { inner, .. } = self;

        let (context, dense, sparse) = inner.into_slice_ptrs_with_context();
        let archetype = (**context).field_descriptors();
        let sparse = unsafe { sparse.as_ref_unchecked() };

        let (entities, bundles) = unsafe { dense.as_ref_unchecked(context) }.into_parts();
        let entities = must_cast_slice(entities);

        (entities, bundles, sparse, archetype)
    }

    #[inline]
    pub fn into_slices(self) -> Slices<'ctx, 'a, T> {
        let (entities, bundles, sparse, _) = self.into_parts();
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
    pub fn into_sparse(self) -> &'a [SparseItem<NoEpochEntity>] {
        let (_, _, sparse) = self.into_slices();
        sparse
    }

    #[inline]
    pub fn as_view(&self) -> ArchetypeStorageView<'_, '_, T> {
        let Self { inner, .. } = self;

        let inner = inner.clone();
        unsafe { ArchetypeStorageView::from_inner(inner) }
    }

    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, T::Meta> {
        let Self { inner, .. } = self;
        (**inner.context()).field_descriptors()
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
    pub fn as_slices_with_archetype(&self) -> SlicesWithArchetype<'_, '_, T> {
        self.as_view().into_parts()
    }

    #[inline]
    pub fn as_slices(&self) -> Slices<'_, '_, T> {
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
    pub fn as_sparse(&self) -> &[SparseItem<NoEpochEntity>] {
        self.as_view().into_sparse()
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { inner, .. } = self;
        unsafe { inner.clone().as_ref_unchecked() }.contains_key(entity.into())
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<ErasedBundleRefs<'_, '_, T>> {
        self.as_view().into_get(entity)
    }

    #[inline]
    pub fn into_get(self, entity: Entity) -> Option<ErasedBundleRefs<'ctx, 'a, T>> {
        let Self { inner, .. } = self;
        unsafe { inner.as_ref_unchecked() }.into_get(entity.into())
    }

    #[inline]
    pub fn as_bundles_with_archetype<B, M>(
        &self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<BundlesWithArchetype<'_, '_, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_view()
            .into_bundles_with_archetype::<B, M>(components)
    }

    #[inline]
    pub fn into_bundles_with_archetype<B, M>(
        self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<BundlesWithArchetype<'ctx, 'a, B, T>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (entities, bundles, sparse, archetype) = self.into_parts();
        archetype.check_compatibility_of::<B, M>(components)?;

        let bundles = bundles
            .downcast::<B, M>(components)
            .map_err(|error| error.source)
            .expect("archetype compatibility should have been already checked");
        Ok((entities, bundles, sparse, archetype))
    }

    #[inline]
    pub fn as_bundles<B, M>(
        &self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<Bundles<'_, '_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_view().into_bundles::<B, M>(components)
    }

    #[inline]
    pub fn into_bundles<B, M>(
        self,
        components: &ComponentRegistryView<impl Sized, M>,
    ) -> Result<Bundles<'ctx, 'a, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (entities, bundles, sparse, _) =
            self.into_bundles_with_archetype::<B, M>(components)?;
        Ok((entities, bundles, sparse))
    }

    #[inline]
    pub fn get_bundle<B, M>(
        &self,
        components: &ComponentRegistryView<impl Sized, M>,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_view().into_get_bundle::<B, M>(components, entity)
    }

    #[inline]
    pub fn into_get_bundle<B, M>(
        self,
        components: &ComponentRegistryView<impl Sized, M>,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'a, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
        M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.archetype()
            .check_compatibility_of::<B, M>(components)?;

        let Some(bundle) = self.into_get(entity) else {
            return Ok(None);
        };
        let bundle = bundle
            .downcast::<B, M>(components)
            .map_err(|error| error.source)
            .expect("archetype compatibility should have been already checked");
        Ok(Some(bundle))
    }
}

impl<T> Debug for ArchetypeStorageView<'_, '_, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let component_ids = &self.archetype().into_component_ids();
        f.debug_struct("ArchetypeStorageView")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}

impl<T> Clone for ArchetypeStorageView<'_, '_, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner, .. } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<T> Copy for ArchetypeStorageView<'_, '_, T>
where
    T: ErasedArchetypeSoa + ?Sized,
    for<'a> T::Archetype<'a>: Copy,
{
}

type SlicesWithArchetype<'ctx, 'a, T> = (
    &'a [Entity],
    ErasedBundles<'ctx, 'a, T>,
    &'a [SparseItem<NoEpochEntity>],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type Slices<'ctx, 'a, T> = (
    &'a [Entity],
    ErasedBundles<'ctx, 'a, T>,
    &'a [SparseItem<NoEpochEntity>],
);

type BundlesWithArchetype<'ctx, 'a, B, T> = (
    &'a [Entity],
    BundleSlices<'a, B>,
    &'a [SparseItem<NoEpochEntity>],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type Bundles<'ctx, 'a, B> = (
    &'a [Entity],
    BundleSlices<'a, B>,
    &'a [SparseItem<NoEpochEntity>],
);
