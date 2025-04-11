use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr,
};

use gpecs_soa_erased::{
    erased::{ErasedSoa, ErasedSoaContext},
    field::{
        ErasedField, ErasedFieldRef, ErasedFieldRefMut, ErasedFieldSlice, ErasedFieldSliceMut,
    },
};
use gpecs_sparse::{error::TryReserveError, key::Key, set::EpochSparseSet};
use indexmap::{set::Iter as IndexSetIter, IndexSet};

use crate::{
    bundle::{error::DuplicateComponentError, Bundle},
    component::registry::{ComponentId, ComponentRegistry},
    entity::Entity,
    soa::traits::FieldDescriptor,
};

use super::{
    erased::{
        from_erased_fields, from_erased_refs, from_erased_refs_mut, from_erased_slices,
        from_erased_slices_mut, get_component_info_fail, into_erased_fields, into_erased_refs,
        into_erased_refs_mut, into_erased_slices, into_erased_slices_mut, ErasedComponents,
    },
    error::{
        ExclusiveComponentError, IncompatibleBundleError, IncompatibleBundleExactError,
        IncompatibleBundleValueError, TooFewComponentsError,
    },
    utils::try_collect_component_ids,
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

type ErasedStorage = EpochSparseSet<NoEpochEntity, ErasedSoa>;

pub struct ArchetypeStorage {
    component_ids: IndexSet<ComponentId>,
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
        let component_ids = try_collect_component_ids(component_ids, IndexSet::insert)?;

        let descriptors = component_ids.iter().map(|&id| {
            let info = components
                .get_info(id)
                .unwrap_or_else(|| get_component_info_fail(&id));
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
    pub fn of<B>(
        components: &mut ComponentRegistry,
        context: &B::Context,
    ) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
    {
        let component_ids = B::component_ids(context, components)?.into_iter().collect();

        let context = ErasedSoaContext::of::<B>(context);
        let erased_storage = ErasedStorage::with_context(context);

        Ok(Self {
            component_ids,
            erased_storage,
        })
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds<'_> {
        let Self { component_ids, .. } = self;
        let inner = component_ids.iter();
        ComponentIds { inner }
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
        components: &mut ComponentRegistry,
        context: &B::Context,
    ) -> Result<B::Slices<'_>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let mut bundle_component_ids = B::component_ids(context, components)?.into_iter();
        if let Some(component_id) = bundle_component_ids.find(|id| !component_ids.contains(id)) {
            return Err(ExclusiveComponentError::new(component_id).into());
        }

        let (len, fields) = ErasedStorageExt::components(erased_storage, components, component_ids);
        let bundle_component_ids = B::component_ids(context, components)
            .expect("components of the bundle should be unique");
        let slices = unsafe {
            from_erased_slices::<B>(components, context, bundle_component_ids, len, fields)
        };
        Ok(slices)
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn components_mut<B>(
        &mut self,
        components: &mut ComponentRegistry,
        context: &B::Context,
    ) -> Result<B::SlicesMut<'_>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let mut bundle_component_ids = B::component_ids(context, components)?.into_iter();
        if let Some(component_id) = bundle_component_ids.find(|id| !component_ids.contains(id)) {
            return Err(ExclusiveComponentError::new(component_id).into());
        }

        let (len, fields) =
            ErasedStorageExt::components_mut(erased_storage, components, component_ids);
        let bundle_component_ids = B::component_ids(context, components)
            .expect("components of the bundle should be unique");
        let slices = unsafe {
            from_erased_slices_mut::<B>(components, context, bundle_component_ids, len, fields)
        };
        Ok(slices)
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn get<B>(
        &self,
        components: &mut ComponentRegistry,
        context: &B::Context,
        entity: Entity,
    ) -> Result<Option<B::Refs<'_>>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let mut bundle_component_ids = B::component_ids(context, components)?.into_iter();
        if let Some(component_id) = bundle_component_ids.find(|id| !component_ids.contains(id)) {
            return Err(ExclusiveComponentError::new(component_id).into());
        }

        let Some(fields) = ErasedStorageExt::get(erased_storage, components, component_ids, entity)
        else {
            return Ok(None);
        };
        let bundle_component_ids = B::component_ids(context, components)
            .expect("components of the bundle should be unique");
        let refs =
            unsafe { from_erased_refs::<B>(components, context, bundle_component_ids, fields) };
        Ok(Some(refs))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn get_mut<B>(
        &mut self,
        components: &mut ComponentRegistry,
        context: &B::Context,
        entity: Entity,
    ) -> Result<Option<B::RefsMut<'_>>, IncompatibleBundleError>
    where
        B: Bundle,
    {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let mut bundle_component_ids = B::component_ids(context, components)?.into_iter();
        if let Some(component_id) = bundle_component_ids.find(|id| !component_ids.contains(id)) {
            return Err(ExclusiveComponentError::new(component_id).into());
        }

        let Some(fields) =
            ErasedStorageExt::get_mut(erased_storage, components, component_ids, entity)
        else {
            return Ok(None);
        };
        let bundle_component_ids = B::component_ids(context, components)
            .expect("components of the bundle should be unique");
        let refs =
            unsafe { from_erased_refs_mut::<B>(components, context, bundle_component_ids, fields) };
        Ok(Some(refs))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn insert<B>(
        &mut self,
        components: &mut ComponentRegistry,
        context: &B::Context,
        entity: Entity,
        value: B,
    ) -> Result<Option<B>, IncompatibleBundleValueError<B>>
    where
        B: Bundle,
    {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let mut bundle_component_ids_count = 0;
        let mut bundle_component_ids = match B::component_ids(context, components) {
            Ok(bundle_component_ids) => bundle_component_ids
                .into_iter()
                .inspect(|_| bundle_component_ids_count += 1),
            Err(error) => {
                let reason = error.into();
                return Err(IncompatibleBundleValueError { value, reason });
            }
        };
        if let Some(component_id) = bundle_component_ids.find(|id| !component_ids.contains(id)) {
            let reason = ExclusiveComponentError::new(component_id).into();
            return Err(IncompatibleBundleValueError { value, reason });
        }

        bundle_component_ids.for_each(drop);
        if bundle_component_ids_count != component_ids.len() {
            let reason = TooFewComponentsError.into();
            return Err(IncompatibleBundleValueError { value, reason });
        }

        let bundle_component_ids = B::component_ids(context, components)
            .expect("components of the bundle should be unique");
        let fields = into_erased_fields::<B>(components, context, bundle_component_ids, value);
        let Some(fields) =
            ErasedStorageExt::insert(erased_storage, components, component_ids, entity, fields)
        else {
            return Ok(None);
        };
        let bundle_component_ids = B::component_ids(context, components)
            .expect("components of the bundle should be unique");
        let value =
            unsafe { from_erased_fields::<B>(components, context, bundle_component_ids, fields) };
        Ok(Some(value))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn remove<B>(
        &mut self,
        components: &mut ComponentRegistry,
        context: &B::Context,
        entity: Entity,
    ) -> Result<Option<B>, IncompatibleBundleExactError>
    where
        B: Bundle,
    {
        let Self {
            ref component_ids,
            erased_storage,
        } = self;

        let mut bundle_component_ids_count = 0;
        let mut bundle_component_ids = B::component_ids(context, components)?
            .into_iter()
            .inspect(|_| bundle_component_ids_count += 1);
        if let Some(component_id) = bundle_component_ids.find(|id| !component_ids.contains(id)) {
            return Err(ExclusiveComponentError::new(component_id).into());
        }

        bundle_component_ids.for_each(drop);
        if bundle_component_ids_count != component_ids.len() {
            return Err(TooFewComponentsError.into());
        }

        let Some(fields) =
            ErasedStorageExt::remove(erased_storage, components, component_ids, entity)
        else {
            return Ok(None);
        };
        let bundle_component_ids = B::component_ids(context, components)
            .expect("components of the bundle should be unique");
        let value =
            unsafe { from_erased_fields::<B>(components, context, bundle_component_ids, fields) };
        Ok(Some(value))
    }

    #[inline]
    #[track_caller]
    pub(super) fn insert_erased(
        &mut self,
        components: &mut ComponentRegistry,
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
        components: &mut ComponentRegistry,
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

        f.debug_struct("ArchetypeStorage")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct ComponentIds<'a> {
    inner: IndexSetIter<'a, ComponentId>,
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
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
    ) -> (usize, ErasedComponents<ErasedFieldSlice<'_>>);

    fn components_mut(
        &mut self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
    ) -> (usize, ErasedComponents<ErasedFieldSliceMut<'_>>);

    fn insert(
        &mut self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
        entity: Entity,
        fields: ErasedComponents<ErasedField>,
    ) -> Option<ErasedComponents<ErasedField>>;

    fn remove(
        &mut self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField>>;

    fn get(
        &self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRef<'_>>>;

    fn get_mut(
        &mut self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRefMut<'_>>>;
}

impl ErasedStorageExt for ErasedStorage {
    #[inline]
    #[allow(unsafe_code)]
    fn entities(&self) -> &[Entity] {
        let entities = ptr::from_ref(self.as_keys_slice()) as *const [_];
        unsafe { &*entities }
    }

    #[inline]
    fn components(
        &self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
    ) -> (usize, ErasedComponents<ErasedFieldSlice<'_>>) {
        let (context, slices) = ErasedStorage::as_view(self).into_slices_with_context();
        let component_ids = component_ids.iter().copied();
        into_erased_slices::<ErasedSoa>(components, context, component_ids, slices)
    }

    #[inline]
    fn components_mut(
        &mut self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
    ) -> (usize, ErasedComponents<ErasedFieldSliceMut<'_>>) {
        let (context, slices) = ErasedStorage::as_mut_view(self).into_slices_with_context();
        let component_ids = component_ids.iter().copied();
        into_erased_slices_mut::<ErasedSoa>(components, context, component_ids, slices)
    }

    #[inline]
    #[allow(unsafe_code)]
    fn insert(
        &mut self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
        entity: Entity,
        fields: ErasedComponents<ErasedField>,
    ) -> Option<ErasedComponents<ErasedField>> {
        let value = unsafe {
            let component_ids = component_ids.iter().copied();
            from_erased_fields::<ErasedSoa>(components, self.context(), component_ids, fields)
        };
        let value = ErasedStorage::insert(self, entity.into(), value)?;

        let component_ids = component_ids.iter().copied();
        let context = self.context();
        let fields = into_erased_fields::<ErasedSoa>(components, context, component_ids, value);
        Some(fields)
    }

    #[inline]
    fn remove(
        &mut self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedField>> {
        let value = ErasedStorage::remove(self, entity.into())?;

        let component_ids = component_ids.iter().copied();
        let context = self.context();
        let fields = into_erased_fields::<ErasedSoa>(components, context, component_ids, value);
        Some(fields)
    }

    #[inline]
    fn get(
        &self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRef<'_>>> {
        let view = ErasedStorage::as_view(self);
        let (context, refs) = view.into_get_with_context(entity.into());

        let component_ids = component_ids.iter().copied();
        let refs = into_erased_refs::<ErasedSoa>(components, context, component_ids, refs?);
        Some(refs)
    }

    #[inline]
    fn get_mut(
        &mut self,
        components: &mut ComponentRegistry,
        component_ids: &IndexSet<ComponentId>,
        entity: Entity,
    ) -> Option<ErasedComponents<ErasedFieldRefMut<'_>>> {
        let view = ErasedStorage::as_mut_view(self);
        let (context, refs) = view.into_get_mut_with_context(entity.into());

        let component_ids = component_ids.iter().copied();
        let refs = into_erased_refs_mut::<ErasedSoa>(components, context, component_ids, refs?);
        Some(refs)
    }
}
