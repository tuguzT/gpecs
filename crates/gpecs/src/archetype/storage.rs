use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr,
};

use gpecs_soa_erased::{
    erased::{ErasedSoa, ErasedSoaContext, ErasedSoaRefsMut},
    field::{
        ErasedField, ErasedFieldRef, ErasedFieldRefMut, ErasedFieldSlice, ErasedFieldSliceMut,
    },
};
use gpecs_sparse::{
    error::TryReserveError,
    key::Key,
    pair::{KeyValueSlices, KeyValueSlicesMut},
    set::EpochSparseSet,
};
use indexmap::{map::Keys, IndexMap, IndexSet};

use crate::{
    archetype::erased::drop_erased_in_place,
    bundle::Bundle,
    component::registry::{ComponentId, ComponentRegistry, DropFn},
    entity::Entity,
    soa::traits::{DefaultContext, FieldDescriptor, Soa},
};

use super::{
    erased::{
        from_erased_fields, from_erased_refs, from_erased_refs_mut, from_erased_slices,
        from_erased_slices_mut, get_component_info_fail, into_erased_fields, into_erased_refs,
        into_erased_refs_mut, into_erased_slices, into_erased_slices_mut, ErasedComponents,
    },
    error::{
        DuplicateComponentError, ExclusiveComponentError, IncompatibleBundleError,
        IncompatibleBundleExactError, IncompatibleBundleValueError, TooFewComponentsError,
    },
    utils::{try_collect_component_ids, try_collect_maybe_component_ids},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
struct NoEpochEntity(Entity);

impl Key for NoEpochEntity {
    type SparseIndex = <Entity as Key>::SparseIndex;
    type Epoch = ();

    fn new(sparse_index: Self::SparseIndex, _: Self::Epoch) -> Self {
        let entity = <Entity as Key>::new(sparse_index, Default::default());
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

pub type Slices<'a, B> = (&'a [Entity], <B as Soa>::Slices<'a>);
pub type SlicesMut<'a, B> = (&'a [Entity], <B as Soa>::SlicesMut<'a>);

type ErasedStorage = EpochSparseSet<NoEpochEntity, ErasedSoa>;
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
        let component_ids = try_collect_component_ids(component_ids, |map, component_id| {
            let info = components
                .get_component_info(component_id)
                .unwrap_or_else(|| get_component_info_fail(&component_id));
            ComponentIdMap::insert(map, component_id, info.drop_fn()).is_none()
        })?;

        let descriptors = component_ids.keys().map(|&component_id| {
            let info = components
                .get_component_info(component_id)
                .unwrap_or_else(|| get_component_info_fail(&component_id));
            info.descriptor()
        });
        let context = ErasedSoaContext::new(descriptors);
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
        let component_ids = try_collect_component_ids(component_ids, |map, component_id| {
            let info = components
                .get_component_info(component_id)
                .unwrap_or_else(|| get_component_info_fail(&component_id));
            ComponentIdMap::insert(map, component_id, info.drop_fn()).is_none()
        })?;

        let context = ErasedSoaContext::of::<B>(&DefaultContext::default());
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
    pub fn bundle_compatibility_of<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<(), IncompatibleBundleError>
    where
        B: Bundle,
    {
        let component_ids = B::get_components(components);
        let component_ids = try_collect_maybe_component_ids(component_ids, IndexSet::<_>::insert)?;
        self.bundle_compatibility_inner(component_ids)
    }

    #[inline]
    pub fn bundle_compatibility<I>(&self, component_ids: I) -> Result<(), IncompatibleBundleError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let component_ids = try_collect_component_ids(component_ids, IndexSet::<_>::insert)?;
        self.bundle_compatibility_inner(component_ids)
    }

    #[inline]
    pub fn bundle_compatibility_of_exact<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<(), IncompatibleBundleExactError>
    where
        B: Bundle,
    {
        let component_ids = B::get_components(components);
        let component_ids = try_collect_maybe_component_ids(component_ids, IndexSet::<_>::insert)?;
        self.bundle_compatibility_exact_inner(component_ids)
    }

    #[inline]
    pub fn bundle_compatibility_exact<I>(
        &self,
        component_ids: I,
    ) -> Result<(), IncompatibleBundleExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let component_ids = try_collect_component_ids(component_ids, IndexSet::<_>::insert)?;
        self.bundle_compatibility_exact_inner(component_ids)
    }

    #[inline]
    fn bundle_compatibility_inner<I>(
        &self,
        bundle_component_ids: I,
    ) -> Result<(), IncompatibleBundleError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let Self { component_ids, .. } = self;

        if let Some(component) = bundle_component_ids
            .into_iter()
            .find(|id| !component_ids.contains_key(id))
        {
            return Err(ExclusiveComponentError::new(component).into());
        }
        Ok(())
    }

    #[inline]
    fn bundle_compatibility_exact_inner<I>(
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
        if let Some(component) = bundle_component_ids.find(|id| !component_ids.contains_key(id)) {
            return Err(ExclusiveComponentError::new(component).into());
        }

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
        ErasedStorageExt::entities(erased_storage)
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { erased_storage, .. } = self;
        erased_storage.contains_key(entity.into())
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn components<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<Slices<B>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        self.bundle_compatibility_of::<B>(components)?;

        let Self {
            component_ids,
            erased_storage,
        } = self;

        let (entities, fields) =
            ErasedStorageExt::components(erased_storage, components, component_ids);
        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let components = unsafe {
            let len = entities.len();
            let context = DefaultContext::default();
            from_erased_slices::<B>(components, &context, bundle_component_ids, len, fields)
        };
        Ok((entities, components))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn components_mut<B>(
        &mut self,
        components: &ComponentRegistry,
    ) -> Result<SlicesMut<B>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        self.bundle_compatibility_of::<B>(components)?;

        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let (entities, fields) =
            ErasedStorageExt::components_mut(erased_storage, components, component_ids);
        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let components = unsafe {
            let len = entities.len();
            let context = DefaultContext::default();
            from_erased_slices_mut::<B>(components, &context, bundle_component_ids, len, fields)
        };
        Ok((entities, components))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn get<B>(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<B::Refs<'_>>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        self.bundle_compatibility_of::<B>(components)?;

        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let Some(fields) = ErasedStorageExt::get(erased_storage, components, component_ids, entity)
        else {
            return Ok(None);
        };
        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let refs = unsafe {
            let context = DefaultContext::default();
            from_erased_refs::<B>(components, &context, bundle_component_ids, fields)
        };
        Ok(Some(refs))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn get_mut<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<B::RefsMut<'_>>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        self.bundle_compatibility_of::<B>(components)?;

        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let Some(fields) =
            ErasedStorageExt::get_mut(erased_storage, components, component_ids, entity)
        else {
            return Ok(None);
        };
        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let refs = unsafe {
            let context = DefaultContext::default();
            from_erased_refs_mut::<B>(components, &context, bundle_component_ids, fields)
        };
        Ok(Some(refs))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn insert<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
        value: B,
    ) -> Result<Option<B>, IncompatibleBundleValueError<B>>
    where
        B: Bundle,
    {
        if let Err(reason) = self.bundle_compatibility_of_exact::<B>(components) {
            return Err(IncompatibleBundleValueError { value, reason });
        }

        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let context = DefaultContext::default();
        let fields = into_erased_fields::<B>(components, &context, bundle_component_ids, value);
        let Some(fields) =
            ErasedStorageExt::insert(erased_storage, components, component_ids, entity, fields)
        else {
            return Ok(None);
        };
        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let value =
            unsafe { from_erased_fields::<B>(components, &context, bundle_component_ids, fields) };
        Ok(Some(value))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn remove<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<B>, IncompatibleBundleExactError>
    where
        B: Bundle,
    {
        self.bundle_compatibility_of_exact::<B>(components)?;

        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let Some(fields) =
            ErasedStorageExt::remove(erased_storage, components, component_ids, entity)
        else {
            return Ok(None);
        };
        let bundle_component_ids = B::get_components(components)
            .into_iter()
            .map(|component_id| component_id.expect("all of components should be registered"));
        let value = unsafe {
            let context = DefaultContext::default();
            from_erased_fields::<B>(components, &context, bundle_component_ids, fields)
        };
        Ok(Some(value))
    }

    #[inline]
    #[track_caller]
    pub fn destroy_in_place(&mut self, entity: Entity) -> bool {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;
        let Some(mut erased_fields) = erased_storage.remove(entity.into()) else {
            return false;
        };

        let erased_refs_mut = erased_fields.as_refs_mut();
        Self::destroy_refs_mut(component_ids, erased_refs_mut);
        true
    }

    #[inline]
    fn destroy_refs_mut(component_ids: &ComponentIdMap, erased_refs_mut: ErasedSoaRefsMut<'_>) {
        let fields = erased_refs_mut.into_field_refs();
        debug_assert_eq!(fields.len(), component_ids.len());

        let fields = fields
            .into_vec()
            .into_iter()
            .zip(component_ids.values().copied());
        #[allow(unsafe_code)]
        unsafe {
            drop_erased_in_place(fields)
        }
    }

    #[inline]
    #[track_caller]
    pub(super) fn insert_erased(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
        fields: ErasedComponents<ErasedField>,
    ) -> Option<ErasedComponents<ErasedField>> {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        ErasedStorageExt::insert(erased_storage, components, component_ids, entity, fields)
    }

    #[inline]
    #[track_caller]
    pub(super) fn remove_erased(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField>> {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        ErasedStorageExt::remove(erased_storage, components, component_ids, entity)
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
            erased_storage,
        } = self;

        erased_storage
            .values_mut()
            .for_each(|erased_refs_mut| Self::destroy_refs_mut(component_ids, erased_refs_mut))
    }
}

#[derive(Clone)]
pub struct ComponentIds<'a> {
    inner: Keys<'a, ComponentId, Option<DropFn>>,
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

    fn components(
        &self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
    ) -> (&[Entity], ErasedComponents<ErasedFieldSlice<'_>>);

    fn components_mut(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
    ) -> (&[Entity], ErasedComponents<ErasedFieldSliceMut<'_>>);

    fn insert(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
        fields: ErasedComponents<ErasedField>,
    ) -> Option<ErasedComponents<ErasedField>>;

    fn remove(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField>>;

    fn get(
        &self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRef<'_>>>;

    fn get_mut(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRefMut<'_>>>;
}

impl ErasedStorageExt for ErasedStorage {
    #[inline]
    #[allow(unsafe_code)]
    fn entities(&self) -> &[Entity] {
        let entities = self.as_keys_slice();
        unsafe { &*(ptr::from_ref(entities) as *const [Entity]) }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn components(
        &self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
    ) -> (&[Entity], ErasedComponents<ErasedFieldSlice<'_>>) {
        let (dense, _) = ErasedStorage::as_view(self).into_parts();
        let (context, KeyValueSlices { keys, values }) = dense.into_slices_with_context();

        let entities = unsafe { &*(ptr::from_ref(keys) as *const [Entity]) };
        let component_ids = component_ids.keys().copied();
        let (len, fields) =
            into_erased_slices::<ErasedSoa>(components, context, component_ids, values);
        if entities.len() != len {
            unreachable!("count of entities should match count of components")
        }
        (entities, fields)
    }

    #[inline]
    #[allow(unsafe_code)]
    fn components_mut(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
    ) -> (&[Entity], ErasedComponents<ErasedFieldSliceMut<'_>>) {
        let (dense, _) = ErasedStorage::as_mut_view(self).into_parts();
        let (context, KeyValueSlicesMut { keys, values }) = dense.into_slices_with_context();

        let entities = unsafe { &*(ptr::from_ref(keys) as *const [Entity]) };
        let component_ids = component_ids.keys().copied();
        let (len, fields) =
            into_erased_slices_mut::<ErasedSoa>(components, context, component_ids, values);
        if entities.len() != len {
            unreachable!("count of entities should match count of components")
        }
        (entities, fields)
    }

    #[inline]
    #[allow(unsafe_code)]
    fn insert(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
        fields: ErasedComponents<ErasedField>,
    ) -> Option<ErasedComponents<ErasedField>> {
        let value = unsafe {
            let component_ids = component_ids.keys().copied();
            from_erased_fields::<ErasedSoa>(components, self.context(), component_ids, fields)
        };
        let value = ErasedStorage::insert(self, entity.into(), value)?;

        let component_ids = component_ids.keys().copied();
        let context = self.context();
        let fields = into_erased_fields::<ErasedSoa>(components, context, component_ids, value);
        Some(fields)
    }

    #[inline]
    fn remove(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField>> {
        let value = ErasedStorage::remove(self, entity.into())?;

        let component_ids = component_ids.keys().copied();
        let context = self.context();
        let fields = into_erased_fields::<ErasedSoa>(components, context, component_ids, value);
        Some(fields)
    }

    #[inline]
    fn get(
        &self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRef<'_>>> {
        let view = ErasedStorage::as_view(self);
        let (context, refs) = view.into_get_with_context(entity.into());

        let component_ids = component_ids.keys().copied();
        let refs = into_erased_refs::<ErasedSoa>(components, context, component_ids, refs?);
        Some(refs)
    }

    #[inline]
    fn get_mut(
        &mut self,
        components: &ComponentRegistry,
        component_ids: &ComponentIdMap,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRefMut<'_>>> {
        let view = ErasedStorage::as_mut_view(self);
        let (context, refs) = view.into_get_mut_with_context(entity.into());

        let component_ids = component_ids.keys().copied();
        let refs = into_erased_refs_mut::<ErasedSoa>(components, context, component_ids, refs?);
        Some(refs)
    }
}
