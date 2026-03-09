use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use bytemuck::{Pod, Zeroable, must_cast_slice};
use gpecs_sparse::{TryInsertAccess, error::TryReserveError, key::Key, set::EpochSparseSet};

use crate::{
    archetype::{
        erased::{ErasedArchetype, ErasedArchetypeIter, FromComponentInfo},
        error::{
            ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError,
            IncompatibleArchetypeExactError, IncompatibleBundleValueError, MissingComponentError,
        },
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut, BundleSlices, BundleSlicesMut,
        erased::{
            ErasedArchetypeKind, ErasedBorrowedBundle, ErasedBundle, ErasedBundleKind,
            ErasedBundleMutRefs, ErasedBundleMutSlices, ErasedBundleRefs, ErasedBundleSlices,
            FromErasedComponent, ShuffledBundle,
        },
    },
    component::{
        erased::ErasedComponent,
        registry::{ComponentId, ComponentInfo, ComponentRegistry, DropFn},
    },
    entity::Entity,
    soa::{
        self,
        field::FieldDescriptor,
        traits::{RawSoaContext, ReadSoaContext, WriteSoaContext},
    },
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct NoEpochEntity(pub Entity);

impl Key for NoEpochEntity {
    type SparseIndex = <Entity as Key>::SparseIndex;
    type Epoch = ();

    #[inline]
    fn new(sparse_index: Self::SparseIndex, (): Self::Epoch) -> Self {
        let epoch = <Entity as Key>::Epoch::default();
        let entity = <Entity as Key>::new(sparse_index, epoch);
        Self(entity)
    }

    #[inline]
    fn sparse_index(self) -> Self::SparseIndex {
        let Self(entity) = self;
        entity.sparse_index()
    }

    #[inline]
    fn epoch(self) -> Self::Epoch {}
}

impl From<Entity> for NoEpochEntity {
    #[inline]
    fn from(entity: Entity) -> Self {
        Self(entity)
    }
}

impl From<NoEpochEntity> for Entity {
    #[inline]
    fn from(entity: NoEpochEntity) -> Self {
        let NoEpochEntity(entity) = entity;
        entity
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StorageMeta {
    descriptor: FieldDescriptor,
    drop_fn: Option<DropFn>,
}

impl AsRef<FieldDescriptor> for StorageMeta {
    #[inline]
    fn as_ref(&self) -> &FieldDescriptor {
        let Self { descriptor, .. } = self;
        descriptor
    }
}

impl AsRef<Option<DropFn>> for StorageMeta {
    #[inline]
    fn as_ref(&self) -> &Option<DropFn> {
        let Self { drop_fn, .. } = self;
        drop_fn
    }
}

impl FromComponentInfo for StorageMeta {
    #[inline]
    fn from_component_info(info: &ComponentInfo) -> Self {
        Self {
            descriptor: info.descriptor(),
            drop_fn: info.drop_fn(),
        }
    }
}

impl FromErasedComponent for StorageMeta {
    #[inline]
    fn from_erased_component(component: &ErasedComponent) -> Self {
        Self {
            descriptor: FieldDescriptor::new(component.as_field().layout()),
            drop_fn: component.drop_fn(),
        }
    }
}

pub struct ArchetypeStorage {
    sparse_set: EpochSparseSet<NoEpochEntity, ErasedBundle<StorageMeta>>,
}

impl ArchetypeStorage {
    #[inline]
    pub fn new<I>(components: &ComponentRegistry, component_ids: I) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = ErasedArchetype::new(components, component_ids)?;
        let sparse_set = EpochSparseSet::with_context(archetype);

        let me = Self { sparse_set };
        Ok(me)
    }

    #[inline]
    pub fn of<B>(components: &ComponentRegistry) -> Result<Self, ArchetypeError>
    where
        B: Bundle,
    {
        let archetype = ErasedArchetype::of::<B>(components)?;
        let sparse_set = EpochSparseSet::with_context(archetype);

        let me = Self { sparse_set };
        Ok(me)
    }

    #[inline]
    pub fn register<B>(components: &mut ComponentRegistry) -> Result<Self, DuplicateComponentError>
    where
        B: Bundle,
    {
        let archetype = ErasedArchetype::register::<B>(components)?;
        let sparse_set = EpochSparseSet::with_context(archetype);

        let me = Self { sparse_set };
        Ok(me)
    }

    #[inline]
    pub fn archetype(&self) -> &ErasedArchetype<StorageMeta> {
        let Self { sparse_set } = self;
        sparse_set.context()
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds<'_> {
        let inner = self.archetype().iter();
        ComponentIds { inner }
    }

    #[inline]
    pub fn check_compatibility(&self, other: &Self) -> Result<(), MissingComponentError> {
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
        let (entities, bundles) = self.as_slices();
        let bundles = bundles.downcast::<B>(components)?;
        Ok((entities, bundles))
    }

    #[inline]
    pub fn bundles_mut<B>(
        &mut self,
        components: &ComponentRegistry,
    ) -> Result<(&[Entity], BundleSlicesMut<'_, B>), IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let (entities, bundles) = self.as_mut_slices();
        let bundles = bundles.downcast::<B>(components)?;
        Ok((entities, bundles))
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
        let Some(bundle) = self.get(entity) else {
            return Ok(None);
        };

        let bundle = bundle.downcast::<B>(components)?;
        Ok(Some(bundle))
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
        let Some(bundle) = self.get_mut(entity) else {
            return Ok(None);
        };

        let bundle = bundle.downcast::<B>(components)?;
        Ok(Some(bundle))
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
            let error = IncompatibleBundleValueError { value, reason };
            return Err(error);
        }

        let Self { sparse_set } = self;
        let bundle = sparse_set.insert_from(entity.into(), |_, access| match access? {
            TryInsertAccess::ReadWrite(dst) => {
                let dst = dst
                    .into_ptrs()
                    .downcast::<B>(components)
                    .expect("exact archetype compatibility should be already checked");
                let value = unsafe { soa::ptr::replace::<B, _, _>(B::CONTEXT, dst, value) };
                Some(value)
            }
            TryInsertAccess::WriteOnly(dst) => {
                let dst = dst
                    .into_inner()
                    .downcast::<B>(components)
                    .expect("exact archetype compatibility should be already checked");
                unsafe { B::CONTEXT.write(dst, value) };
                None
            }
        });
        Ok(bundle)
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

        let Self { sparse_set } = self;
        let bundle = sparse_set.swap_remove_into(entity.into(), |_, src| {
            let src = src?
                .cast_const()
                .downcast::<B>(components)
                .expect("exact archetype compatibility should be already checked");
            let bundle = unsafe { B::CONTEXT.read(src) };
            Some(bundle)
        });
        Ok(bundle)
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<ErasedBundleRefs<'_, '_, StorageMeta>> {
        let Self { sparse_set } = self;
        sparse_set.get(entity.into())
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<ErasedBundleMutRefs<'_, '_, StorageMeta>> {
        let Self { sparse_set } = self;
        sparse_set.get_mut(entity.into())
    }

    #[inline]
    pub fn insert<T>(
        &mut self,
        entity: Entity,
        bundle: ErasedBundleKind<T>,
    ) -> Result<Option<ErasedBorrowedBundle<'_, StorageMeta>>, IncompatibleArchetypeExactError>
    where
        T: ErasedArchetypeKind<Meta = StorageMeta>,
    {
        let archetype = self.archetype();
        archetype.check_exact_compatibility(bundle.archetype())?;

        let entity = entity.into();
        let bundle = bundle
            .shuffle(archetype.clone())
            .expect("exact archetype compatibility should have been already checked");

        let Self { sparse_set } = self;
        let bundle = match bundle {
            ShuffledBundle::Original(bundle) => sparse_set.insert(entity, bundle),
            ShuffledBundle::Other(bundle) => sparse_set.insert(entity, bundle),
        };
        Ok(bundle)
    }

    #[inline]
    pub fn remove(&mut self, entity: Entity) -> Option<ErasedBorrowedBundle<'_, StorageMeta>> {
        let Self { sparse_set } = self;
        sparse_set.swap_remove(entity.into())
    }

    #[inline]
    pub fn destroy(&mut self, entity: Entity) -> bool {
        let Self { sparse_set } = self;

        sparse_set.swap_remove_into(entity.into(), |archetype, ptrs| {
            let Some(ptrs) = ptrs else { return false };
            unsafe { archetype.ptrs_drop_in_place(ptrs) };
            true
        })
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], ErasedBundleSlices<'_, '_, StorageMeta>) {
        let Self { sparse_set } = self;

        let (dense, _) = sparse_set.as_view().into_parts();
        let (entities, slices) = dense.into_slices().into_parts();

        let entities = must_cast_slice(entities);
        (entities, slices)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&[Entity], ErasedBundleMutSlices<'_, '_, StorageMeta>) {
        let Self { sparse_set } = self;

        let (dense, _) = sparse_set.as_mut_view().into_parts();
        let (entities, slices) = dense.into_slices().into_parts();

        let entities = must_cast_slice(entities);
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

#[derive(Clone)]
pub struct ComponentIds<'a> {
    inner: ErasedArchetypeIter<'a, StorageMeta>,
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
