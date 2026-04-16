use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use bytemuck::must_cast_slice_mut;
use gpecs_sparse::{item::SparseItem, view::EpochSparseViewMutPtr};

use crate::{
    archetype::{
        erased::ErasedArchetypeView,
        storage::{ArchetypeStorageView, NoEpochEntity, traits::ErasedArchetypeSoa},
    },
    entity::Entity,
    soa::{
        field::FieldDescriptors,
        traits::{
            Refs as ErasedBundleRefs, RefsMut as ErasedBundleRefsMut, Slices as ErasedBundles,
            SlicesMut as ErasedBundlesMut,
        },
    },
};

type Inner<'a, T> = EpochSparseViewMutPtr<'a, NoEpochEntity, T>;

#[repr(transparent)]
pub struct ArchetypeStorageViewMut<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    inner: Inner<'ctx, T>,
    phantom: PhantomData<fn() -> &'a mut ()>,
}

impl<'ctx, 'a, T> ArchetypeStorageViewMut<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: Inner<'ctx, T>) -> Self {
        let phantom = PhantomData;
        Self { inner, phantom }
    }

    #[inline]
    pub unsafe fn into_parts(self) -> MutSlicesWithArchetype<'ctx, 'a, T> {
        let Self { inner, .. } = self;

        let (context, dense, sparse) = inner.into_mut_slice_ptrs_with_context();
        let archetype = (**context).field_descriptors();
        let sparse = unsafe { sparse.as_mut_unchecked() };

        let (entities, bundles) = unsafe { dense.as_mut_unchecked(context) }.into_parts();
        let entities = must_cast_slice_mut(entities);

        (entities, bundles, sparse, archetype)
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
    pub fn as_slices(&self) -> Slices<'_, T> {
        let (entities, bundles, sparse, _) = self.as_slices_with_archetype();
        (entities, bundles, sparse)
    }

    #[inline]
    pub fn as_entities(&self) -> &[Entity] {
        let (entities, _, _) = self.as_slices();
        entities
    }

    #[inline]
    pub fn as_erased_bundles(&self) -> ErasedBundles<'_, '_, T> {
        let (_, bundles, _) = self.as_slices();
        bundles
    }

    #[inline]
    pub fn as_sparse(&self) -> &[SparseItem<NoEpochEntity>] {
        let (_, _, sparse) = self.as_slices();
        sparse
    }

    #[inline]
    pub unsafe fn as_mut_slices_with_archetype(&mut self) -> MutSlicesWithArchetype<'_, '_, T> {
        let Self { inner, .. } = self;

        let (context, dense, sparse) = inner.as_mut_slice_ptrs_with_context();
        let archetype = (**context).field_descriptors();
        let sparse = unsafe { sparse.as_mut_unchecked() };

        let (entities, bundles) = unsafe { dense.as_mut_unchecked(context) }.into_parts();
        let entities = must_cast_slice_mut(entities);

        (entities, bundles, sparse, archetype)
    }

    #[inline]
    pub unsafe fn as_mut_slices(&mut self) -> MutSlices<'_, T> {
        let (entities, bundles, sparse, _) = unsafe { self.as_mut_slices_with_archetype() };
        (entities, bundles, sparse)
    }

    #[inline]
    pub fn as_mut_erased_bundles(&mut self) -> ErasedBundlesMut<'_, '_, T> {
        let (_, bundles, _) = unsafe { self.as_mut_slices() };
        bundles
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
        let Self { inner, .. } = self;
        unsafe { inner.as_ref_unchecked() }.into_get(entity.into())
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

type SlicesWithArchetype<'ctx, 'a, T> = (
    &'a [Entity],
    ErasedBundles<'ctx, 'a, T>,
    &'a [SparseItem<NoEpochEntity>],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type Slices<'a, T> = (
    &'a [Entity],
    ErasedBundles<'a, 'a, T>,
    &'a [SparseItem<NoEpochEntity>],
);

type MutSlicesWithArchetype<'ctx, 'a, T> = (
    &'a mut [Entity],
    ErasedBundlesMut<'ctx, 'a, T>,
    &'a mut [SparseItem<NoEpochEntity>],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type MutSlices<'a, T> = (
    &'a mut [Entity],
    ErasedBundlesMut<'a, 'a, T>,
    &'a mut [SparseItem<NoEpochEntity>],
);
