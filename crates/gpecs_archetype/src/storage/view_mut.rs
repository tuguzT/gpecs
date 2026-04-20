use core::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use bytemuck::must_cast_slice_mut;
use gpecs_entity::Entity;
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
    item::{DenseSlicesMut, SparseItem},
    view::{EpochSparseViewMut, EpochSparseViewMutPtr},
};

use crate::{
    erased::ErasedArchetypeView,
    storage::{ArchetypeStorageView, NoEpochEntity, traits::ErasedArchetypeSoa},
};

type Inner<'a, T> = EpochSparseViewMutPtr<'a, NoEpochEntity, T>;

#[repr(transparent)]
pub struct ArchetypeStorageViewMut<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    inner: Inner<'ctx, T>,
    phantom: PhantomData<&'a mut [PtrsItem<T::Ptrs>]>,
}

impl<'ctx, 'a, T> ArchetypeStorageViewMut<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    pub fn new(
        context: &'ctx T::Context,
        entities: &'a mut [Entity],
        bundles: ErasedBundlesMut<'ctx, 'a, T>,
        sparse: &'a mut [SparseItem<NoEpochEntity>],
    ) -> Result<Self, FromPartsError<NoEpochEntity>> {
        let entities = must_cast_slice_mut(entities);
        let dense = SoaSlicesMut::new(
            Identity::from_inner_ref(context),
            DenseSlicesMut::new(context, entities, bundles),
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
        sparse: &'a mut [SparseItem<NoEpochEntity>],
    ) -> Self {
        let entities = must_cast_slice_mut(entities);
        let dense = SoaSlicesMut::new(
            Identity::from_inner_ref(context),
            DenseSlicesMut::new(context, entities, bundles),
        );

        let inner = unsafe { EpochSparseViewMut::from_parts(dense, sparse) }.into_mut_view_ptr();
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub(crate) unsafe fn from_inner(inner: Inner<'ctx, T>) -> Self {
        let phantom = PhantomData;
        Self { inner, phantom }
    }

    #[inline]
    pub unsafe fn into_parts(self) -> MutSlicesWithArchetype<'ctx, 'a, T> {
        let Self { inner, .. } = self;

        let (context, dense, sparse) = inner.into_mut_slice_ptrs_with_context();
        let archetype = (**context).field_layouts();
        let sparse = unsafe { sparse.as_mut_unchecked() };

        let (entities, bundles) = unsafe { dense.as_mut_unchecked(context) }.into_parts();
        let entities = must_cast_slice_mut(entities);

        (entities, bundles, sparse, archetype)
    }

    #[inline]
    pub fn into_slices(self) -> Slices<'ctx, 'a, T> {
        let (entities, bundles, sparse, _) = unsafe { self.into_parts() };
        (entities, bundles.into(), sparse)
    }

    #[inline]
    pub unsafe fn into_mut_slices(self) -> MutSlices<'ctx, 'a, T> {
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
    pub fn into_sparse(self) -> &'a [SparseItem<NoEpochEntity>] {
        let (_, _, sparse) = self.into_slices();
        sparse
    }

    #[inline]
    pub fn as_view(&self) -> ArchetypeStorageView<'_, '_, T> {
        let Self { inner, .. } = self;

        let inner = inner.clone().cast_const();
        unsafe { ArchetypeStorageView::from_inner(inner) }
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> ArchetypeStorageViewMut<'_, '_, T> {
        let Self { inner, .. } = self;

        let inner = inner.clone();
        unsafe { ArchetypeStorageViewMut::from_inner(inner) }
    }

    #[inline]
    pub fn into_view(self) -> ArchetypeStorageView<'ctx, 'a, T> {
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
    pub unsafe fn as_mut_slices_with_archetype(&mut self) -> MutSlicesWithArchetype<'_, '_, T> {
        unsafe { self.as_mut_view().into_parts() }
    }

    #[inline]
    pub unsafe fn as_mut_slices(&mut self) -> MutSlices<'_, '_, T> {
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
}

impl<T> Debug for ArchetypeStorageViewMut<'_, '_, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let component_ids = &self.archetype().into_component_ids();
        f.debug_struct("ArchetypeStorageViewMut")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}

impl<'ctx, 'a, T> From<ArchetypeStorageViewMut<'ctx, 'a, T>> for ArchetypeStorageView<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    fn from(view: ArchetypeStorageViewMut<'ctx, 'a, T>) -> Self {
        view.into_view()
    }
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

type MutSlicesWithArchetype<'ctx, 'a, T> = (
    &'a mut [Entity],
    ErasedBundlesMut<'ctx, 'a, T>,
    &'a mut [SparseItem<NoEpochEntity>],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type MutSlices<'ctx, 'a, T> = (
    &'a mut [Entity],
    ErasedBundlesMut<'ctx, 'a, T>,
    &'a mut [SparseItem<NoEpochEntity>],
);
