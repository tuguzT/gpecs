use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use bytemuck::{Pod, Zeroable, must_cast_slice};
use gpecs_soa_erased::{BoxedErasedSoa, ErasedSoaContext};
use gpecs_sparse::{error::TryReserveError, key::Key, set::EpochSparseSet};
use indexmap::map::Keys as IndexMapKeys;
use itertools::{Itertools, zip_eq};

use crate::{
    bundle::Bundle,
    component::{
        erased::{
            ErasedComponent, ErasedComponentMutRef, ErasedComponentMutSlice, ErasedComponentRef,
            ErasedComponentSlice,
        },
        registry::{ComponentId, ComponentRegistry, DropFn},
    },
    entity::Entity,
    hash::{IndexMap, IndexSet},
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        traits::{Refs, RefsMut, Slices, SlicesMut, SoaContext},
    },
};

use super::{
    collect::{try_collect_component_ids, try_collect_maybe_component_ids},
    erased::{
        ErasedBundle, from_erased_fields, from_erased_mut_slices, from_erased_refs,
        from_erased_refs_mut, from_erased_slices, get_component_info_fail, into_erased_fields,
        validate_components,
    },
    error::{
        DuplicateComponentError, IncompatibleBundleError, IncompatibleBundleExactError,
        IncompatibleBundleValueError, MissingComponentError, TooFewComponentsError,
    },
};

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

pub type Bundles<'a, B> = (&'a [Entity], Slices<'a, 'a, B>);
pub type BundlesMut<'a, B> = (&'a [Entity], SlicesMut<'a, 'a, B>);

type ErasedStorage = EpochSparseSet<NoEpochEntity, ErasedBundle>;
type ComponentIdMap = IndexMap<ComponentId, Option<DropFn>>;

pub struct ArchetypeStorage {
    component_ids: ComponentIdMap,
    erased_storage: ErasedStorage,
}

impl ArchetypeStorage {
    #[inline]
    #[track_caller]
    pub fn new<I>(
        components: &ComponentRegistry,
        component_ids: I,
    ) -> Result<Self, DuplicateComponentError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let component_ids = try_collect_component_ids(
            component_ids,
            |map, component_id| {
                let info = components
                    .get_component_info(component_id)
                    .unwrap_or_else(|| get_component_info_fail(component_id));
                ComponentIdMap::insert(map, component_id, info.drop_fn()).is_none()
            },
            Clone::clone,
        )?;

        let descriptors = component_ids
            .keys()
            .map(|&component_id| {
                let info = components
                    .get_component_info(component_id)
                    .unwrap_or_else(|| get_component_info_fail(component_id));
                info.descriptor()
            })
            .collect();
        let context = ErasedSoaContext::new(descriptors).expect("descriptors should be valid");
        let erased_storage = ErasedStorage::with_context(context);

        Ok(Self {
            component_ids,
            erased_storage,
        })
    }

    #[inline]
    pub fn of<B>(components: &mut ComponentRegistry) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
    {
        let component_ids = B::register_components(components);
        let component_ids = try_collect_component_ids(
            component_ids,
            |map, component_id| {
                let info = components
                    .get_component_info(component_id)
                    .unwrap_or_else(|| get_component_info_fail(component_id));
                ComponentIdMap::insert(map, component_id, info.drop_fn()).is_none()
            },
            Clone::clone,
        )?;

        let context = ErasedSoaContext::of::<B>(B::CONTEXT).expect("descriptors should be valid");
        let erased_storage = ErasedStorage::with_context(context);

        Ok(Self {
            component_ids,
            erased_storage,
        })
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds<'_> {
        let Self { component_ids, .. } = self;
        let inner = component_ids.keys();
        ComponentIds { inner }
    }

    #[inline]
    pub fn bundle_compatibility<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<(), IncompatibleBundleError>
    where
        B: Bundle,
    {
        let component_ids = B::get_components(components);
        let component_ids =
            try_collect_maybe_component_ids(component_ids, IndexSet::<_>::insert, Clone::clone)?;
        self.components_compatibility_inner(component_ids)
    }

    #[inline]
    pub fn components_compatibility<I>(
        &self,
        component_ids: I,
    ) -> Result<(), IncompatibleBundleError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let component_ids =
            try_collect_component_ids(component_ids, IndexSet::<_>::insert, Clone::clone)?;
        self.components_compatibility_inner(component_ids)
    }

    #[inline]
    pub fn bundle_compatibility_exact<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<(), IncompatibleBundleExactError>
    where
        B: Bundle,
    {
        let component_ids = B::get_components(components);
        let component_ids =
            try_collect_maybe_component_ids(component_ids, IndexSet::<_>::insert, Clone::clone)?;
        self.components_compatibility_exact_inner(component_ids)
    }

    #[inline]
    pub fn components_compatibility_exact<I>(
        &self,
        component_ids: I,
    ) -> Result<(), IncompatibleBundleExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let component_ids =
            try_collect_component_ids(component_ids, IndexSet::<_>::insert, Clone::clone)?;
        self.components_compatibility_exact_inner(component_ids)
    }

    #[inline]
    fn components_compatibility_inner<I>(
        &self,
        bundle_component_ids: I,
    ) -> Result<(), IncompatibleBundleError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let Self { component_ids, .. } = self;

        if let Some(component_id) = bundle_component_ids
            .into_iter()
            .find(|id| !component_ids.contains_key(id))
        {
            let error = MissingComponentError::new(component_id);
            return Err(error.into());
        }
        Ok(())
    }

    #[inline]
    fn components_compatibility_exact_inner<I>(
        &self,
        bundle_component_ids: I,
    ) -> Result<(), IncompatibleBundleExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let Self { component_ids, .. } = self;

        let mut bundle_component_ids_count = 0;
        let mut bundle_component_ids = bundle_component_ids
            .into_iter()
            .inspect(|_| bundle_component_ids_count += 1);
        self.components_compatibility_inner(bundle_component_ids.by_ref())?;

        bundle_component_ids.for_each(drop);
        if bundle_component_ids_count != component_ids.len() {
            return Err(TooFewComponentsError.into());
        }

        Ok(())
    }

    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { erased_storage, .. } = self;
        erased_storage.context().field_descriptors()
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
    ) -> Result<Bundles<'_, B>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        self.bundle_compatibility::<B>(components)?;

        let Self {
            component_ids,
            erased_storage,
        } = self;

        let (entities, fields) = erased_storage.erased_components(components, component_ids);
        let components = unsafe { from_erased_slices::<B>(components, entities.len(), fields) };
        let components = B::Context::upcast_slices(components);
        Ok((entities, components))
    }

    #[inline]
    pub fn bundles_mut<B>(
        &mut self,
        components: &ComponentRegistry,
    ) -> Result<BundlesMut<'_, B>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        self.bundle_compatibility::<B>(components)?;

        let Self {
            ref component_ids,
            ref mut erased_storage,
        } = *self;

        let (entities, fields) = erased_storage.erased_components_mut(components, component_ids);
        let components = unsafe { from_erased_mut_slices::<B>(components, entities.len(), fields) };
        let components = B::Context::upcast_mut_slices(components);
        Ok((entities, components))
    }

    #[inline]
    pub fn get_bundle<B>(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<Refs<'_, '_, B>>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        self.bundle_compatibility::<B>(components)?;

        let Self {
            component_ids,
            erased_storage,
        } = self;

        let Some(fields) = erased_storage.get_erased(components, component_ids, entity) else {
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
    ) -> Result<Option<RefsMut<'_, '_, B>>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        self.bundle_compatibility::<B>(components)?;

        let Self {
            ref component_ids,
            ref mut erased_storage,
        } = *self;

        let Some(fields) = erased_storage.get_erased_mut(components, component_ids, entity) else {
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
        if let Err(reason) = self.bundle_compatibility_exact::<B>(components) {
            return Err(IncompatibleBundleValueError { value, reason });
        }

        let Self {
            ref component_ids,
            ref mut erased_storage,
        } = *self;

        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let fields =
            unsafe { into_erased_fields::<B>(components, B::CONTEXT, bundle_component_ids, value) };
        let Some(fields) = erased_storage.insert_erased(components, component_ids, entity, fields)
        else {
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
    ) -> Result<Option<B>, IncompatibleBundleExactError>
    where
        B: Bundle,
    {
        self.bundle_compatibility_exact::<B>(components)?;

        let Self {
            ref component_ids,
            ref mut erased_storage,
        } = *self;

        let Some(fields) = erased_storage.remove_erased(components, component_ids, entity) else {
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
    #[track_caller]
    pub fn destroy_in_place(&mut self, entity: Entity) -> bool {
        let Self {
            ref component_ids,
            ref mut erased_storage,
        } = *self;

        let Some(erased_fields) = erased_storage.swap_remove(entity.into()) else {
            return false;
        };

        Self::drop_erased(component_ids, erased_fields);
        true
    }

    #[inline]
    fn drop_erased(component_ids: &ComponentIdMap, mut erased_fields: ErasedBundle) {
        let fields = erased_fields.as_mut_fields();
        let component_ids = component_ids.values().copied();
        for (mut field, drop_fn) in zip_eq(fields, component_ids) {
            let Some(drop_fn) = drop_fn else { continue };
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
        let Self {
            ref component_ids,
            ref mut erased_storage,
        } = *self;
        erased_storage.insert_erased(components, component_ids, entity, fields)
    }

    #[inline]
    #[track_caller]
    pub(super) fn remove_erased(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponent>> {
        let Self {
            ref component_ids,
            ref mut erased_storage,
        } = *self;
        erased_storage.remove_erased(components, component_ids, entity)
    }

    #[inline]
    #[track_caller]
    pub(crate) fn erased_components(
        &self,
        components: &ComponentRegistry,
    ) -> (&[Entity], IndexSet<ErasedComponentSlice<'_>>) {
        let Self {
            component_ids,
            erased_storage,
        } = self;
        erased_storage.erased_components(components, component_ids)
    }

    #[inline]
    #[track_caller]
    #[expect(dead_code)]
    pub(crate) fn erased_components_mut(
        &mut self,
        components: &ComponentRegistry,
    ) -> (&[Entity], IndexSet<ErasedComponentMutSlice<'_>>) {
        let Self {
            component_ids,
            erased_storage,
        } = self;
        erased_storage.erased_components_mut(components, component_ids)
    }
}

impl Debug for ArchetypeStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { component_ids, .. } = self;

        let component_ids = &component_ids.keys();
        f.debug_struct("ArchetypeStorage")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}

impl Drop for ArchetypeStorage {
    fn drop(&mut self) {
        let Self {
            ref component_ids,
            ref mut erased_storage,
        } = *self;

        erased_storage
            .drain()
            .for_each(|(_, erased_fields)| Self::drop_erased(component_ids, erased_fields));
    }
}

#[derive(Clone)]
pub struct ComponentIds<'a> {
    inner: IndexMapKeys<'a, ComponentId, Option<DropFn>>,
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
        inner.next().copied()
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
        inner.nth(n).copied()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().copied()
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.copied().collect()
    }
}

impl DoubleEndedIterator for ComponentIds<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().copied()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).copied()
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
        component_ids: &ComponentIdMap,
    ) -> (&[Entity], IndexSet<ErasedComponentSlice<'_>>);

    fn erased_components_mut(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
    ) -> (&[Entity], IndexSet<ErasedComponentMutSlice<'_>>);

    fn insert_erased(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
        fields: IndexSet<ErasedComponent>,
    ) -> Option<IndexSet<ErasedComponent>>;

    fn remove_erased(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponent>>;

    fn get_erased(
        &self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponentRef<'_>>>;

    fn get_erased_mut(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
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
        component_ids: &ComponentIdMap,
    ) -> (&[Entity], IndexSet<ErasedComponentSlice<'_>>) {
        let (dense, _) = Self::as_view(self).into_parts();
        let (context, slices) = dense.into_slices_with_context();
        let (entities, values) = slices.into_parts();

        let entities = must_cast_slice(entities);
        let component_ids = component_ids.keys().copied();
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
        component_ids: &ComponentIdMap,
    ) -> (&[Entity], IndexSet<ErasedComponentMutSlice<'_>>) {
        let (dense, _) = Self::as_mut_view(self).into_parts();
        let (context, slices) = dense.into_slices_with_context();
        let (entities, values) = slices.into_parts();

        let entities = must_cast_slice(entities);
        let component_ids = component_ids.keys().copied();
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
        component_ids: &ComponentIdMap,
        entity: Entity,
        fields: IndexSet<ErasedComponent>,
    ) -> Option<IndexSet<ErasedComponent>> {
        let value = unsafe {
            let context = self.context();
            let component_ids = component_ids.keys().copied();
            from_erased_fields(components, context, component_ids, fields)
        };
        let value = Self::insert(self, entity.into(), value)?;

        let component_ids = component_ids.keys().copied();
        let context = self.context();
        let fields = unsafe { into_erased_fields(components, context, component_ids, value) };
        Some(fields)
    }

    #[inline]
    fn remove_erased(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponent>> {
        let value = Self::swap_remove(self, entity.into())?;

        let component_ids = component_ids.keys().copied();
        let context = self.context();
        let fields = unsafe { into_erased_fields(components, context, component_ids, value) };
        Some(fields)
    }

    #[inline]
    fn get_erased(
        &self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponentRef<'_>>> {
        let view = Self::as_view(self);
        let (context, refs) = view.into_get_with_context(entity.into());

        let component_ids = component_ids.keys().copied();
        let refs = validate_components::<BoxedErasedSoa<_>, _>(components, context, component_ids)
            .zip_eq(refs?)
            .map(|(id, r#ref)| unsafe { ErasedComponentRef::from_parts(id, r#ref) })
            .collect();
        Some(refs)
    }

    #[inline]
    fn get_erased_mut(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<IndexSet<ErasedComponentMutRef<'_>>> {
        let view = Self::as_mut_view(self);
        let (context, refs) = view.into_get_mut_with_context(entity.into());

        let component_ids = component_ids.keys().copied();
        let refs = validate_components::<BoxedErasedSoa<_>, _>(components, context, component_ids)
            .zip_eq(refs?)
            .map(|(id, r#ref)| unsafe { ErasedComponentMutRef::from_parts(id, r#ref) })
            .collect();
        Some(refs)
    }
}
