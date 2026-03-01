use std::{
    borrow::Borrow,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use bytemuck::{Pod, Zeroable, must_cast_slice};
use gpecs_soa_erased::{
    ErasedSoa, ErasedSoaContext, ptr::slice::CoreSliceItemPtrs, storage::BoxedAlignedUninitStorage,
};
use gpecs_sparse::{error::TryReserveError, key::Key, set::EpochSparseSet};

use crate::{
    archetype::{
        erased::{ErasedArchetype, ErasedArchetypeIter, FromComponentInfo},
        error::{
            ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError,
            IncompatibleArchetypeExactError, IncompatibleBundleValueError,
        },
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut, BundleSlices, BundleSlicesMut,
        erased::{
            ErasedBorrowedBundle, ErasedBundle, ErasedBundleMutRefs, ErasedBundleMutSlices,
            ErasedBundleRefs, ErasedBundleSlices,
        },
    },
    component::{
        erased::ErasedComponent,
        registry::{ComponentId, ComponentInfo, ComponentRegistry, DropFn},
    },
    entity::Entity,
    hash::IndexSet,
    soa::field::FieldDescriptor,
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

#[derive(Debug, Clone, Copy)]
pub struct ErasedStorageMeta {
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

impl AsRef<Option<DropFn>> for ErasedStorageMeta {
    #[inline]
    fn as_ref(&self) -> &Option<DropFn> {
        let Self { drop_fn, .. } = self;
        drop_fn
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

type ErasedBundleRaw = ErasedSoa<
    BoxedAlignedUninitStorage,
    ErasedArchetype<ErasedStorageMeta>,
    CoreSliceItemPtrs<MaybeUninit<u8>>,
>;

type ErasedReadBundleRaw<'a> = ErasedSoa<
    BoxedAlignedUninitStorage,
    &'a ErasedArchetype<ErasedStorageMeta>,
    CoreSliceItemPtrs<MaybeUninit<u8>>,
>;

pub struct ArchetypeStorage {
    sparse_set: EpochSparseSet<NoEpochEntity, ErasedBundleRaw>,
}

impl ArchetypeStorage {
    #[inline]
    pub fn new<I>(components: &ComponentRegistry, component_ids: I) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = ErasedArchetype::new(components, component_ids)?;

        let context = ErasedSoaContext::new(archetype).expect("descriptors should be valid");
        let sparse_set = EpochSparseSet::with_context(context);

        let me = Self { sparse_set };
        Ok(me)
    }

    #[inline]
    pub fn of<B>(components: &mut ComponentRegistry) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
    {
        let archetype = ErasedArchetype::of::<B>(components)?;

        let context = ErasedSoaContext::new(archetype).expect("descriptors should be valid");
        let sparse_set = EpochSparseSet::with_context(context);

        let me = Self { sparse_set };
        Ok(me)
    }

    #[inline]
    pub fn archetype(&self) -> &ErasedArchetype<ErasedStorageMeta> {
        let Self { sparse_set } = self;
        sparse_set.context().as_inner()
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds<'_> {
        let inner = self.archetype().iter();
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
        let Self { sparse_set } = self;
        sparse_set.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { sparse_set } = self;
        sparse_set.is_empty()
    }

    #[inline]
    pub fn sparse_len(&self) -> usize {
        let Self { sparse_set } = self;
        sparse_set.sparse_len()
    }

    #[inline]
    pub fn sparse_is_empty(&self) -> bool {
        let Self { sparse_set } = self;
        sparse_set.sparse_is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { sparse_set } = self;
        sparse_set.capacity()
    }

    #[inline]
    pub fn sparse_capacity(&self) -> usize {
        let Self { sparse_set } = self;
        sparse_set.sparse_capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self { sparse_set } = self;
        sparse_set.reserve(additional_dense, additional_sparse);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self { sparse_set } = self;
        sparse_set.reserve_exact(additional_dense, additional_sparse);
    }

    #[inline]
    pub fn try_reserve(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self { sparse_set } = self;
        sparse_set.try_reserve(additional_dense, additional_sparse)
    }

    #[inline]
    pub fn try_reserve_exact(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self { sparse_set } = self;
        sparse_set.try_reserve_exact(additional_dense, additional_sparse)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        let Self { sparse_set } = self;
        sparse_set.shrink_to_fit();
    }

    #[inline]
    pub fn dense_shrink_to_fit(&mut self) {
        let Self { sparse_set } = self;
        sparse_set.dense_shrink_to_fit();
    }

    #[inline]
    pub fn sparse_shrink_to_fit(&mut self) {
        let Self { sparse_set } = self;
        sparse_set.sparse_shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let Self { sparse_set } = self;
        sparse_set.shrink_to(min_capacity);
    }

    #[inline]
    pub fn dense_shrink_to(&mut self, min_capacity: usize) {
        let Self { sparse_set } = self;
        sparse_set.dense_shrink_to(min_capacity);
    }

    #[inline]
    pub fn sparse_shrink_to(&mut self, min_capacity: usize) {
        let Self { sparse_set } = self;
        sparse_set.sparse_shrink_to(min_capacity);
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        let Self { sparse_set } = self;

        let entities = sparse_set.as_key_slice();
        must_cast_slice(entities)
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let Self { sparse_set } = self;
        sparse_set.contains_key(entity.into())
    }

    #[inline]
    pub fn bundles<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<(&[Entity], BundleSlices<'_, B>), IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let (entities, fields) = self.erased_components();
        let components = fields.downcast::<B>(components)?;
        Ok((entities, components))
    }

    #[inline]
    pub fn bundles_mut<B>(
        &mut self,
        components: &ComponentRegistry,
    ) -> Result<(&[Entity], BundleSlicesMut<'_, B>), IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.check_compatibility_of::<B>(components)?;

        let (entities, fields) = self.erased_components_mut();
        let components = fields.downcast::<B>(components)?;
        Ok((entities, components))
    }

    #[inline]
    pub fn get_bundle<B>(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let Some(fields) = self.get_erased(entity) else {
            return Ok(None);
        };

        let refs = fields.downcast::<B>(components)?;
        Ok(Some(refs))
    }

    #[inline]
    pub fn get_bundle_mut<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<BundleRefsMut<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        self.check_compatibility_of::<B>(components)?;

        let Some(fields) = self.get_erased_mut(entity) else {
            return Ok(None);
        };

        let refs = fields.downcast::<B>(components)?;
        Ok(Some(refs))
    }

    #[inline]
    pub fn insert_bundle<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        value: B,
    ) -> Result<Option<B>, IncompatibleBundleValueError<B>>
    where
        B: Bundle,
    {
        if let Err(reason) = self.check_exact_compatibility_of::<B>(components) {
            return Err(IncompatibleBundleValueError { value, reason });
        }

        let fields = ErasedBundle::<ErasedStorageMeta>::try_from(components, value)
            .map_err(|error| error.reason)
            .expect("bundle compatibility should have been already checked");
        let fields = fields
            .into_iter()
            .map(|component| component.expect("component should be allocated successfully"))
            .collect();

        let fields = self
            .insert_erased(entity, fields)
            .expect("bundle compatibility should have been already checked");
        let Some(fields) = fields else {
            return Ok(None);
        };

        let value = fields
            .downcast(components)
            .expect("exact archetype compatibility should be already checked");
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

        let Some(fields) = self.remove_erased(entity) else {
            return Ok(None);
        };

        let value = fields
            .downcast(components)
            .expect("exact archetype compatibility should be already checked");
        Ok(Some(value))
    }

    #[inline]
    pub fn destroy_in_place(&mut self, entity: Entity) -> bool {
        let Self { sparse_set } = self;

        let Some(erased_fields) = sparse_set.swap_remove(entity.into()) else {
            return false;
        };

        let _ = unsafe { ErasedBundle::from_inner(erased_fields) };
        true
    }

    #[inline]
    #[track_caller]
    pub(super) fn get_erased(
        &self,
        entity: Entity,
    ) -> Option<ErasedBundleRefs<'_, '_, ErasedStorageMeta>> {
        let Self { sparse_set } = self;

        let refs = sparse_set.as_view().into_get(entity.into());
        let refs = unsafe { ErasedBundleRefs::from_inner(refs?) };
        Some(refs)
    }

    #[inline]
    #[track_caller]
    pub(super) fn get_erased_mut(
        &mut self,
        entity: Entity,
    ) -> Option<ErasedBundleMutRefs<'_, '_, ErasedStorageMeta>> {
        let Self { sparse_set } = self;

        let refs = sparse_set.as_mut_view().into_get_mut(entity.into());
        let refs = unsafe { ErasedBundleMutRefs::from_inner(refs?) };
        Some(refs)
    }

    #[inline]
    #[track_caller]
    pub(super) fn insert_erased(
        &mut self,
        entity: Entity,
        fields: IndexSet<ErasedComponent>,
    ) -> Result<Option<ErasedBorrowedBundle<'_, ErasedStorageMeta>>, IncompatibleArchetypeExactError>
    {
        let component_ids = fields.iter().map(ErasedComponent::component_id);
        self.check_exact_compatibility_for(component_ids)?;

        let Self { sparse_set } = self;

        // TODO: if order of components is the same, write them without any reordering
        let value: ErasedReadBundleRaw = {
            let descriptors = sparse_set.context().as_inner();
            let order = descriptors.iter().map(From::from);
            let fields = reorder_fields(fields, order).map(ErasedComponent::into_field);
            ErasedSoa::try_from_fields_descriptors(fields, descriptors)
                .expect("all the fields should be valid")
        };
        let (value, _) = value.into_parts();
        let Some(inner) = sparse_set.insert::<ErasedReadBundleRaw, _>(entity.into(), value) else {
            return Ok(None);
        };

        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(Some(bundle))
    }

    #[inline]
    #[track_caller]
    pub(super) fn remove_erased(
        &mut self,
        entity: Entity,
    ) -> Option<ErasedBorrowedBundle<'_, ErasedStorageMeta>> {
        let Self { sparse_set } = self;

        let inner = sparse_set.swap_remove(entity.into())?;
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Some(bundle)
    }

    #[inline]
    #[track_caller]
    pub(crate) fn erased_components(
        &self,
    ) -> (&[Entity], ErasedBundleSlices<'_, '_, ErasedStorageMeta>) {
        let Self { sparse_set } = self;

        let (dense, _) = sparse_set.as_view().into_parts();
        let (entities, values) = dense.into_slices().into_parts();

        let entities = must_cast_slice(entities);
        let slices = unsafe { ErasedBundleSlices::from_inner(values) };
        (entities, slices)
    }

    #[inline]
    #[track_caller]
    pub(crate) fn erased_components_mut(
        &mut self,
    ) -> (&[Entity], ErasedBundleMutSlices<'_, '_, ErasedStorageMeta>) {
        let Self { sparse_set } = self;

        let (dense, _) = sparse_set.as_mut_view().into_parts();
        let (entities, values) = dense.into_slices().into_parts();

        let entities = must_cast_slice(entities);
        let slices = unsafe { ErasedBundleMutSlices::from_inner(values) };
        (entities, slices)
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
        let Self { sparse_set } = self;

        for (_, erased_fields) in sparse_set.drain() {
            let _ = unsafe { ErasedBundle::from_inner(erased_fields) };
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
        inner.next().map(From::from)
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
        inner.nth(n).map(From::from)
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(From::from)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.map(From::from).collect()
    }
}

impl DoubleEndedIterator for ComponentIds<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(From::from)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(From::from)
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

#[inline]
#[track_caller]
pub fn reorder_fields<I, F>(mut fields: IndexSet<F>, order: I) -> impl Iterator<Item = F>
where
    I: IntoIterator<Item = ComponentId>,
    F: Borrow<ComponentId>,
{
    #[cold]
    #[track_caller]
    #[inline(never)]
    fn remove_field_fail(component_id: ComponentId) -> ! {
        panic!("field of {component_id} should be present")
    }

    order.into_iter().map(move |id| {
        fields
            .swap_take(&id)
            .unwrap_or_else(|| remove_field_fail(id))
    })
}
