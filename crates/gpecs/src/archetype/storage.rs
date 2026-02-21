use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use bytemuck::{Pod, Zeroable, must_cast_slice};
use gpecs_soa_erased::{
    ErasedSoa, ErasedSoaContext, ptr::slice::CoreSliceItemPtrs, storage::BoxedAlignedUninitStorage,
};
use gpecs_sparse::{error::TryReserveError, key::Key, set::EpochSparseSet};
use itertools::{Itertools, zip_eq};

use crate::{
    archetype::{
        erased::{ErasedArchetype, ErasedArchetypeIter, FromComponentInfo},
        error::{
            ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError,
            IncompatibleArchetypeExactError, IncompatibleBundleValueError,
        },
    },
    bundle::{
        Bundle,
        erased::utils::{
            from_erased_fields, from_erased_mut_slices, from_erased_refs, from_erased_refs_mut,
            from_erased_slices, into_erased_fields, validate_components,
        },
    },
    component::{
        erased::{
            ErasedComponent, ErasedComponentMutRef, ErasedComponentMutSlice, ErasedComponentRef,
            ErasedComponentSlice,
        },
        registry::{ComponentId, ComponentInfo, ComponentRegistry, DropFn},
    },
    entity::Entity,
    hash::IndexSet,
    soa::{
        field::FieldDescriptor,
        traits::{Refs, RefsMut, Slices, SlicesMut, SoaContext},
    },
};

pub type Bundles<'a, B> = (&'a [Entity], Slices<'a, 'a, B>);
pub type BundlesMut<'a, B> = (&'a [Entity], SlicesMut<'a, 'a, B>);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
struct NoEpochEntity(Entity);

impl Key for NoEpochEntity {
    type SparseIndex = <Entity as Key>::SparseIndex;
    type Epoch = ();

    fn new(sparse_index: Self::SparseIndex, (): Self::Epoch) -> Self {
        let epoch = <Entity as Key>::Epoch::default();
        let entity = <Entity as Key>::new(sparse_index, epoch);
        Self(entity)
    }

    fn sparse_index(self) -> Self::SparseIndex {
        let Self(entity) = self;
        entity.sparse_index()
    }

    fn epoch(self) -> Self::Epoch {}
}

impl From<Entity> for NoEpochEntity {
    fn from(entity: Entity) -> Self {
        Self(entity)
    }
}

impl From<NoEpochEntity> for Entity {
    fn from(entity: NoEpochEntity) -> Self {
        let NoEpochEntity(entity) = entity;
        entity
    }
}

#[derive(Debug, Clone, Copy)]
struct ErasedStorageMeta {
    descriptor: FieldDescriptor,
    drop_fn: Option<DropFn>,
}

impl AsRef<FieldDescriptor> for ErasedStorageMeta {
    #[inline]
    fn as_ref(&self) -> &FieldDescriptor {
        let Self { descriptor, .. } = self;
        descriptor
    }
}

impl FromComponentInfo for ErasedStorageMeta {
    #[inline]
    fn from_component_info(info: &ComponentInfo) -> Self {
        Self {
            descriptor: info.descriptor(),
            drop_fn: info.drop_fn(),
        }
    }
}

type ErasedBundle = ErasedSoa<
    BoxedAlignedUninitStorage,
    ErasedArchetype<ErasedStorageMeta>,
    CoreSliceItemPtrs<MaybeUninit<u8>>,
>;
type ErasedStorage = EpochSparseSet<NoEpochEntity, ErasedBundle>;

pub struct ArchetypeStorage {
    erased_storage: ErasedStorage,
}

impl ArchetypeStorage {
    #[inline]
    pub fn new<I>(components: &ComponentRegistry, component_ids: I) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = ErasedArchetype::new(components, component_ids)?;

        let context = ErasedSoaContext::new(archetype).expect("descriptors should be valid");
        let erased_storage = ErasedStorage::with_context(context);

        let me = Self { erased_storage };
        Ok(me)
    }

    #[inline]
    pub fn of<B>(components: &mut ComponentRegistry) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
    {
        let archetype = ErasedArchetype::of::<B>(components)?;

        let context = ErasedSoaContext::new(archetype).expect("descriptors should be valid");
        let erased_storage = ErasedStorage::with_context(context);

        let me = Self { erased_storage };
        Ok(me)
    }

    #[inline]
    pub fn archetype(&self) -> &ErasedArchetype<impl FromComponentInfo> {
        let Self { erased_storage } = self;
        erased_storage.context().as_inner()
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds<'_> {
        let Self { erased_storage } = self;

        let archetype = erased_storage.context().as_inner();
        let inner = archetype.iter();
        ComponentIds { inner }
    }

    #[inline]
    pub fn check_compatibility(&self, other: &Self) -> Result<(), IncompatibleArchetypeError> {
        let archetype = self.archetype();
        let other = other.archetype();
        archetype.check_compatibility(other)
    }

    #[inline]
    pub fn check_compatibility_for<I>(
        &self,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = self.archetype();
        archetype.check_compatibility_for(component_ids)
    }

    #[inline]
    pub fn check_compatibility_of<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let archetype = self.archetype();
        archetype.check_compatibility_of::<B>(components)
    }

    #[inline]
    pub fn check_exact_compatibility(
        &self,
        other: &Self,
    ) -> Result<(), IncompatibleArchetypeExactError> {
        let archetype = self.archetype();
        let other = other.archetype();
        archetype.check_exact_compatibility(other)
    }

    #[inline]
    pub fn check_exact_compatibility_for<I>(
        &self,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = self.archetype();
        archetype.check_exact_compatibility_for(component_ids)
    }

    #[inline]
    pub fn check_exact_compatibility_of<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        B: Bundle,
    {
        let archetype = self.archetype();
        archetype.check_exact_compatibility_of::<B>(components)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { erased_storage, .. } = self;
        erased_storage.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { erased_storage, .. } = self;
        erased_storage.is_empty()
    }

    #[inline]
    pub fn sparse_len(&self) -> usize {
        let Self { erased_storage, .. } = self;
        erased_storage.sparse_len()
    }

    #[inline]
    pub fn sparse_is_empty(&self) -> bool {
        let Self { erased_storage, .. } = self;
        erased_storage.sparse_is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { erased_storage, .. } = self;
        erased_storage.capacity()
    }

    #[inline]
    pub fn sparse_capacity(&self) -> usize {
        let Self { erased_storage, .. } = self;
        erased_storage.sparse_capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self { erased_storage, .. } = self;
        erased_storage.reserve(additional_dense, additional_sparse);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self { erased_storage, .. } = self;
        erased_storage.reserve_exact(additional_dense, additional_sparse);
    }

    #[inline]
    pub fn try_reserve(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self { erased_storage, .. } = self;
        erased_storage.try_reserve(additional_dense, additional_sparse)
    }

    #[inline]
    pub fn try_reserve_exact(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self { erased_storage, .. } = self;
        erased_storage.try_reserve_exact(additional_dense, additional_sparse)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        let Self { erased_storage, .. } = self;
        erased_storage.shrink_to_fit();
    }

    #[inline]
    pub fn dense_shrink_to_fit(&mut self) {
        let Self { erased_storage, .. } = self;
        erased_storage.dense_shrink_to_fit();
    }

    #[inline]
    pub fn sparse_shrink_to_fit(&mut self) {
        let Self { erased_storage, .. } = self;
        erased_storage.sparse_shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let Self { erased_storage, .. } = self;
        erased_storage.shrink_to(min_capacity);
    }

    #[inline]
    pub fn dense_shrink_to(&mut self, min_capacity: usize) {
        let Self { erased_storage, .. } = self;
        erased_storage.dense_shrink_to(min_capacity);
    }

    #[inline]
    pub fn sparse_shrink_to(&mut self, min_capacity: usize) {
        let Self { erased_storage, .. } = self;
        erased_storage.sparse_shrink_to(min_capacity);
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        let Self { erased_storage, .. } = self;
        erased_storage.entities()
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { erased_storage, .. } = self;
        erased_storage.contains_key(entity.into())
    }

    #[inline]
    pub fn bundles<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<Bundles<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.check_compatibility_of::<B>(components)?;

        let Self { erased_storage } = self;

        let (entities, fields) = erased_storage.erased_components(components);
        let components = unsafe { from_erased_slices::<B>(components, entities.len(), fields) };
        let components = B::Context::upcast_slices(components);
        Ok((entities, components))
    }

    #[inline]
    pub fn bundles_mut<B>(
        &mut self,
        components: &ComponentRegistry,
    ) -> Result<BundlesMut<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.check_compatibility_of::<B>(components)?;

        let Self { erased_storage } = self;

        let (entities, fields) = erased_storage.erased_components_mut(components);
        let components = unsafe { from_erased_mut_slices::<B>(components, entities.len(), fields) };
        let components = B::Context::upcast_mut_slices(components);
        Ok((entities, components))
    }

    #[inline]
    pub fn get_bundle<B>(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<Refs<'_, '_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.check_compatibility_of::<B>(components)?;

        let Self { erased_storage } = self;
        let Some(fields) = erased_storage.get_erased(components, entity) else {
            return Ok(None);
        };

        let refs = unsafe { from_erased_refs::<B>(components, fields) };
        let refs = B::Context::upcast_refs(refs);
        Ok(Some(refs))
    }

    #[inline]
    pub fn get_bundle_mut<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<RefsMut<'_, '_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.check_compatibility_of::<B>(components)?;

        let Self { erased_storage } = self;
        let Some(fields) = erased_storage.get_erased_mut(components, entity) else {
            return Ok(None);
        };

        let refs = unsafe { from_erased_refs_mut::<B>(components, fields) };
        let refs = B::Context::upcast_mut_refs(refs);
        Ok(Some(refs))
    }

    #[inline]
    pub fn insert_bundle<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
        value: B,
    ) -> Result<Option<B>, IncompatibleBundleValueError<B>>
    where
        B: Bundle,
    {
        if let Err(reason) = self.check_exact_compatibility_of::<B>(components) {
            return Err(IncompatibleBundleValueError { value, reason });
        }

        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let fields =
            unsafe { into_erased_fields::<B>(components, B::CONTEXT, bundle_component_ids, value) };

        let Self { erased_storage } = self;
        let Some(fields) = erased_storage.insert_erased(components, entity, fields) else {
            return Ok(None);
        };

        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let value = unsafe {
            from_erased_fields::<B>(components, B::CONTEXT, bundle_component_ids, fields)
        };
        Ok(Some(value))
    }

    #[inline]
    pub fn remove_bundle<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<B>, IncompatibleArchetypeExactError>
    where
        B: Bundle,
    {
        self.check_exact_compatibility_of::<B>(components)?;

        let Self { erased_storage } = self;
        let Some(fields) = erased_storage.remove_erased(components, entity) else {
            return Ok(None);
        };

        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let value = unsafe {
            from_erased_fields::<B>(components, B::CONTEXT, bundle_component_ids, fields)
        };
        Ok(Some(value))
    }

    #[inline]
    pub fn destroy_in_place(&mut self, entity: Entity) -> bool {
        let Self { erased_storage } = self;

        let Some(erased_fields) = erased_storage.swap_remove(entity.into()) else {
            return false;
        };

        let archetype = erased_storage.context().as_inner();
        unsafe { Self::drop_erased(archetype, erased_fields) }
        true
    }

    #[inline]
    unsafe fn drop_erased(
        archetype: &ErasedArchetype<ErasedStorageMeta>,
        mut erased_fields: ErasedBundle,
    ) {
        for (mut field, (_, meta)) in zip_eq(erased_fields.as_mut_fields(), archetype) {
            let Some(drop_fn) = meta.drop_fn else {
                continue;
            };
            unsafe { drop_fn(field.as_mut_ptr()) }
        }
    }

    #[inline]
    #[track_caller]
    pub(super) fn insert_erased(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
        fields: IndexSet<ErasedComponent>,
    ) -> Option<IndexSet<ErasedComponent>> {
        let Self { erased_storage } = self;
        erased_storage.insert_erased(components, entity, fields)
    }

    #[inline]
    #[track_caller]
    pub(super) fn remove_erased(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponent>> {
        let Self { erased_storage } = self;
        erased_storage.remove_erased(components, entity)
    }

    #[inline]
    #[track_caller]
    pub(crate) fn erased_components(
        &self,
        components: &ComponentRegistry,
    ) -> (&[Entity], IndexSet<ErasedComponentSlice<'_>>) {
        let Self { erased_storage } = self;
        erased_storage.erased_components(components)
    }

    #[inline]
    #[track_caller]
    #[expect(dead_code)]
    pub(crate) fn erased_components_mut(
        &mut self,
        components: &ComponentRegistry,
    ) -> (&[Entity], IndexSet<ErasedComponentMutSlice<'_>>) {
        let Self { erased_storage } = self;
        erased_storage.erased_components_mut(components)
    }
}

impl Debug for ArchetypeStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let component_ids = &self.component_ids();
        f.debug_struct("ArchetypeStorage")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}

impl Drop for ArchetypeStorage {
    fn drop(&mut self) {
        let Self { erased_storage } = self;

        let mut drain = erased_storage.drain();
        while let Some((_, erased_fields)) = drain.next() {
            let archetype = drain.context().as_inner();
            unsafe { Self::drop_erased(archetype, erased_fields) }
        }
    }
}

#[derive(Clone)]
pub struct ComponentIds<'a> {
    inner: ErasedArchetypeIter<'a, ErasedStorageMeta>,
}

impl Debug for ComponentIds<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        Debug::fmt(inner, f)
    }
}

impl Iterator for ComponentIds<'_> {
    type Item = ComponentId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(component_id, _)| component_id)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(component_id, _)| component_id)
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|(component_id, _)| component_id)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.map(|(component_id, _)| component_id).collect()
    }
}

impl DoubleEndedIterator for ComponentIds<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(component_id, _)| component_id)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|(component_id, _)| component_id)
    }
}

impl ExactSizeIterator for ComponentIds<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for ComponentIds<'_> {}

trait ErasedStorageExt {
    fn entities(&self) -> &[Entity];

    fn erased_components(
        &self,
        components: &ComponentRegistry,
    ) -> (&[Entity], IndexSet<ErasedComponentSlice<'_>>);

    fn erased_components_mut(
        &mut self,
        components: &ComponentRegistry,
    ) -> (&[Entity], IndexSet<ErasedComponentMutSlice<'_>>);

    fn insert_erased(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
        fields: IndexSet<ErasedComponent>,
    ) -> Option<IndexSet<ErasedComponent>>;

    fn remove_erased(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponent>>;

    fn get_erased(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponentRef<'_>>>;

    fn get_erased_mut(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponentMutRef<'_>>>;
}

impl ErasedStorageExt for ErasedStorage {
    #[inline]
    fn entities(&self) -> &[Entity] {
        let entities = self.as_key_slice();
        must_cast_slice(entities)
    }

    #[inline]
    fn erased_components(
        &self,
        components: &ComponentRegistry,
    ) -> (&[Entity], IndexSet<ErasedComponentSlice<'_>>) {
        let (dense, _) = Self::as_view(self).into_parts();
        let (context, slices) = dense.into_slices_with_context();
        let (entities, values) = slices.into_parts();

        let entities = must_cast_slice(entities);

        let component_ids = context
            .as_inner()
            .as_inner()
            .iter()
            .map(|(component_id, _)| component_id);
        let fields = validate_components::<ErasedBundle, _>(components, context, component_ids)
            .zip_eq(values)
            .map(|(id, slice)| unsafe { ErasedComponentSlice::from_parts(id, slice) })
            .collect();

        (entities, fields)
    }

    #[inline]
    fn erased_components_mut(
        &mut self,
        components: &ComponentRegistry,
    ) -> (&[Entity], IndexSet<ErasedComponentMutSlice<'_>>) {
        let (dense, _) = Self::as_mut_view(self).into_parts();
        let (context, slices) = dense.into_slices_with_context();
        let (entities, values) = slices.into_parts();

        let entities = must_cast_slice(entities);

        let component_ids = context
            .as_inner()
            .as_inner()
            .iter()
            .map(|(component_id, _)| component_id);
        let fields = validate_components::<ErasedBundle, _>(components, context, component_ids)
            .zip_eq(values)
            .map(|(id, slice)| unsafe { ErasedComponentMutSlice::from_parts(id, slice) })
            .collect();

        (entities, fields)
    }

    #[inline]
    fn insert_erased(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
        fields: IndexSet<ErasedComponent>,
    ) -> Option<IndexSet<ErasedComponent>> {
        let value = unsafe {
            let context = self.context();
            let component_ids = context
                .as_inner()
                .iter()
                .map(|(component_id, _)| component_id);
            from_erased_fields(components, context, component_ids, fields)
        };
        let value = Self::insert(self, entity.into(), value)?;

        let context = self.context();
        let component_ids = context
            .as_inner()
            .iter()
            .map(|(component_id, _)| component_id);
        let fields = unsafe { into_erased_fields(components, context, component_ids, value) };
        Some(fields)
    }

    #[inline]
    fn remove_erased(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponent>> {
        let value = Self::swap_remove(self, entity.into())?;

        let context = self.context();
        let component_ids = context
            .as_inner()
            .iter()
            .map(|(component_id, _)| component_id);
        let fields = unsafe { into_erased_fields(components, context, component_ids, value) };
        Some(fields)
    }

    #[inline]
    fn get_erased(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponentRef<'_>>> {
        let view = Self::as_view(self);
        let (context, refs) = view.into_get_with_context(entity.into());

        let component_ids = context
            .as_inner()
            .iter()
            .map(|(component_id, _)| component_id);
        let refs = validate_components::<ErasedBundle, _>(components, context, component_ids)
            .zip_eq(refs?)
            .map(|(id, r#ref)| unsafe { ErasedComponentRef::from_parts(id, r#ref) })
            .collect();
        Some(refs)
    }

    #[inline]
    fn get_erased_mut(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponentMutRef<'_>>> {
        let view = Self::as_mut_view(self);
        let (context, refs) = view.into_get_mut_with_context(entity.into());

        let component_ids = context
            .as_inner()
            .iter()
            .map(|(component_id, _)| component_id);
        let refs = validate_components::<ErasedBundle, _>(components, context, component_ids)
            .zip_eq(refs?)
            .map(|(id, r#ref)| unsafe { ErasedComponentMutRef::from_parts(id, r#ref) })
            .collect();
        Some(refs)
    }
}
