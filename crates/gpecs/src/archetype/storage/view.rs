use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use bytemuck::must_cast_slice;
use gpecs_sparse::{
    item::SparseItem,
    soa::{
        field::FieldDescriptors,
        traits::{Refs, Slices as Bundles},
    },
    view::EpochSparseViewPtr,
};

use crate::{
    archetype::{
        erased::ErasedArchetypeView,
        storage::{NoEpochEntity, traits::ErasedArchetypeSoa},
    },
    entity::Entity,
};

type Inner<'a, T> = EpochSparseViewPtr<'a, NoEpochEntity, T>;

#[repr(transparent)]
pub struct ArchetypeStorageView<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    inner: Inner<'ctx, T>,
    phantom: PhantomData<fn() -> &'a ()>,
}

impl<'ctx, 'a, T> ArchetypeStorageView<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
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
        let sparse = unsafe { &*sparse };

        let (entities, bundles) = unsafe { dense.deref(context) }.into_parts();
        let entities = must_cast_slice(entities);

        (entities, bundles, sparse, archetype)
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
        self.clone().into_parts()
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
    pub fn as_bundles(&self) -> Bundles<'_, '_, T> {
        let (_, bundles, _) = self.as_slices();
        bundles
    }

    #[inline]
    pub fn as_sparse(&self) -> &[SparseItem<NoEpochEntity>] {
        let (_, _, sparse) = self.as_slices();
        sparse
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { inner, .. } = self;
        unsafe { inner.clone().deref() }.contains_key(entity.into())
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<Refs<'_, '_, T>> {
        self.clone().into_get(entity)
    }

    #[inline]
    pub fn into_get(self, entity: Entity) -> Option<Refs<'ctx, 'a, T>> {
        let Self { inner, .. } = self;
        unsafe { inner.clone().deref() }.into_get(entity.into())
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
    Bundles<'ctx, 'a, T>,
    &'a [SparseItem<NoEpochEntity>],
    ErasedArchetypeView<'ctx, <T as ErasedArchetypeSoa>::Meta>,
);
type Slices<'a, T> = (
    &'a [Entity],
    Bundles<'a, 'a, T>,
    &'a [SparseItem<NoEpochEntity>],
);
