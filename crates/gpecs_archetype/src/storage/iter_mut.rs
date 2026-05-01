use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use bytemuck::must_cast_slice;
use gpecs_entity::Entity;
use gpecs_soa_erased::{
    ptr::slice::PtrsItem,
    soa::{
        field::FieldLayouts,
        traits::{MutPtrs, Ptrs, RefsMut, SliceMutPtrs, SlicePtrs, Slices, SlicesMut},
    },
};

use crate::{
    erased::ErasedArchetypeView,
    storage::{ErasedArchetypeSoa, NoEpochEntity},
};

type Inner<'ctx, T> = gpecs_sparse::iter::RawIterMut<'ctx, NoEpochEntity, T>;

#[repr(transparent)]
pub struct IterMut<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    inner: Inner<'ctx, T>,
    phantom: PhantomData<&'a mut [PtrsItem<T::Ptrs>]>,
}

impl<'ctx, 'a, T> IterMut<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    pub(super) fn from_inner(inner: Inner<'ctx, T>) -> Self {
        let phantom = PhantomData;
        Self { inner, phantom }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, T::Meta> {
        let Self { inner, .. } = self;
        (**inner.context()).field_layouts()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const Entity, Ptrs<'_, T>) {
        let (_, entity, bundle) = self.as_ptrs_with_archetype();
        (entity, bundle)
    }

    #[inline]
    pub fn as_ptrs_with_archetype(
        &self,
    ) -> (ErasedArchetypeView<'_, T::Meta>, *const Entity, Ptrs<'_, T>) {
        let Self { inner, .. } = self;

        let (context, entity, bundle) = inner.as_ptrs_with_context();
        let archetype = (**context).field_layouts();
        let entity = entity.cast();
        (archetype, entity, bundle)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*const Entity, MutPtrs<'ctx, T>) {
        let (_, entity, bundle) = self.as_mut_ptrs_with_archetype();
        (entity, bundle)
    }

    #[inline]
    pub fn as_mut_ptrs_with_archetype(
        &mut self,
    ) -> (
        ErasedArchetypeView<'_, T::Meta>,
        *const Entity,
        MutPtrs<'ctx, T>,
    ) {
        let Self { inner, .. } = self;

        let (context, entity, bundle) = inner.as_mut_ptrs_with_context();
        let archetype = (**context).field_layouts();
        let entity = entity.cast();
        (archetype, entity, bundle)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [Entity], SlicePtrs<'_, T>) {
        let (_, entities, bundles) = self.as_slice_ptrs_with_archetype();
        (entities, bundles)
    }

    #[inline]
    pub fn as_slice_ptrs_with_archetype(
        &self,
    ) -> (
        ErasedArchetypeView<'_, T::Meta>,
        *const [Entity],
        SlicePtrs<'_, T>,
    ) {
        let Self { inner, .. } = self;

        let (context, entities, bundles) = inner.as_slice_ptrs_with_context();
        let archetype = (**context).field_layouts();
        let entities = entities as *const [_];
        (archetype, entities, bundles)
    }

    #[inline]
    pub fn as_mut_slice_ptrs(&mut self) -> (*const [Entity], SliceMutPtrs<'ctx, T>) {
        let (_, entities, bundles) = self.as_mut_slice_ptrs_with_archetype();
        (entities, bundles)
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_archetype(
        &mut self,
    ) -> (
        ErasedArchetypeView<'_, T::Meta>,
        *const [Entity],
        SliceMutPtrs<'ctx, T>,
    ) {
        let Self { inner, .. } = self;

        let (context, entities, bundles) = inner.as_mut_slice_ptrs_with_context();
        let archetype = (**context).field_layouts();
        let entities = entities as *const [_];
        (archetype, entities, bundles)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], Slices<'_, '_, T>) {
        let (_, entities, bundles) = self.as_slices_with_archetype();
        (entities, bundles)
    }

    #[inline]
    pub fn as_slices_with_archetype(
        &self,
    ) -> (
        ErasedArchetypeView<'_, T::Meta>,
        &[Entity],
        Slices<'_, '_, T>,
    ) {
        let Self { inner, .. } = self;

        let inner = unsafe { inner.clone().as_ref_unchecked() };
        let (context, entities, bundles) = inner.into_slices_with_context();

        let archetype = (**context).field_layouts();
        let entities = must_cast_slice(entities);
        (archetype, entities, bundles)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&[Entity], SlicesMut<'_, '_, T>) {
        let (_, entities, bundles) = self.as_mut_slices_with_archetype();
        (entities, bundles)
    }

    #[inline]
    pub fn as_mut_slices_with_archetype(
        &mut self,
    ) -> (
        ErasedArchetypeView<'_, T::Meta>,
        &[Entity],
        SlicesMut<'_, '_, T>,
    ) {
        let Self { inner, .. } = self;

        let inner = unsafe { inner.clone().as_mut_unchecked() };
        let (context, entities, bundles) = inner.into_slices_with_context();

        let archetype = (**context).field_layouts();
        let entities = must_cast_slice(entities);
        (archetype, entities, bundles)
    }

    #[inline]
    fn map_item(item: (*mut NoEpochEntity, MutPtrs<'ctx, T>)) -> (Entity, RefsMut<'ctx, 'a, T>) {
        let (entity, bundle) = item;
        let entity = unsafe { *entity.as_ref_unchecked() }.into();
        let bundle = unsafe { bundle.as_mut_unchecked() };
        (entity, bundle)
    }
}

impl<T> Debug for IterMut<'_, '_, T>
where
    T: ErasedArchetypeSoa + ?Sized,
    for<'a> T::Archetype<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, bundles) = &self.as_slices();
        f.debug_struct("Iter")
            .field("entities", entities)
            .field("bundles", bundles)
            .finish()
    }
}

impl<T> Clone for IterMut<'_, '_, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner, .. } = self;

        let inner = inner.clone();
        Self::from_inner(inner)
    }
}

impl<'ctx, 'a, T> Iterator for IterMut<'ctx, 'a, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    type Item = (Entity, RefsMut<'ctx, 'a, T>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next().map(Self::map_item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.nth(n).map(Self::map_item)
    }
}

impl<T> DoubleEndedIterator for IterMut<'_, '_, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next_back().map(Self::map_item)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.nth_back(n).map(Self::map_item)
    }
}

impl<T> ExactSizeIterator for IterMut<'_, '_, T>
where
    T: ErasedArchetypeSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<T> FusedIterator for IterMut<'_, '_, T> where T: ErasedArchetypeSoa + ?Sized {}
