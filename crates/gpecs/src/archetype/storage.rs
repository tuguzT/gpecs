use std::fmt::{self, Debug};

use bytemuck::{Pod, Zeroable, must_cast_slice};
use gpecs_sparse::{TryInsertAccess, error::TryReserveError, key::Key, set::EpochSparseSet};

use crate::{
    archetype::{
        erased::{
            ErasedArchetype, FromComponentInfo,
            error::{
                ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError,
                IncompatibleArchetypeExactError,
            },
        },
        error::IncompatibleBundleValueError,
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut, BundleSlices, BundleSlicesMut, NewBundle,
        erased::{
            ErasedBorrowedBundle, ErasedBundle, ErasedBundleKind, ErasedBundleMutRefs,
            ErasedBundleMutSlices, ErasedBundleRefs, ErasedBundleSlices, FromErasedComponent,
            ShuffledBundle, traits::ErasedArchetypeKind,
        },
    },
    component::{
        erased::{ErasedComponent, ErasedDrop, WithErasedDrop},
        registry::{
            ComponentId, ComponentInfo, ComponentRegistry, ComponentRegistryView,
            traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType},
        },
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
pub struct ErasedDropMeta {
    desc: FieldDescriptor,
    erased_drop: Option<ErasedDrop>,
}

impl AsRef<FieldDescriptor> for ErasedDropMeta {
    #[inline]
    fn as_ref(&self) -> &FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }
}

impl WithErasedDrop for ErasedDropMeta {
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        let Self { erased_drop, .. } = *self;
        erased_drop
    }
}

impl<Meta> FromComponentInfo<'_, Meta> for ErasedDropMeta
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop,
{
    #[inline]
    fn from_component_info(info: ComponentInfo<&Meta>) -> Self {
        let desc = FromComponentInfo::from_component_info(info);
        let erased_drop = FromComponentInfo::from_component_info(info);
        Self { desc, erased_drop }
    }
}

impl FromErasedComponent for ErasedDropMeta {
    #[inline]
    fn from_erased_component(component: &ErasedComponent) -> Self {
        Self {
            desc: FieldDescriptor::new(component.as_field().layout()),
            erased_drop: component.erased_drop(),
        }
    }
}

pub struct ArchetypeStorage {
    sparse_set: EpochSparseSet<NoEpochEntity, ErasedBundle<ErasedDropMeta>>,
}

impl ArchetypeStorage {
    #[inline]
    pub fn new<I, T>(
        components: &ComponentRegistryView<T, impl ?Sized>,
        component_ids: I,
    ) -> Result<Self, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
        T: AsRef<FieldDescriptor> + WithErasedDrop,
    {
        let archetype = ErasedArchetype::new(components, component_ids)?;
        let me = Self::from_archetype(archetype);
        Ok(me)
    }

    #[inline]
    pub fn of<B, M, T>(components: &ComponentRegistryView<M, T>) -> Result<Self, ArchetypeError>
    where
        B: Bundle,
        M: AsRef<FieldDescriptor> + WithErasedDrop,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let archetype = ErasedArchetype::of::<B, M, T>(components)?;
        let me = Self::from_archetype(archetype);
        Ok(me)
    }

    #[inline]
    pub fn register<B, M, T>(
        components: &mut ComponentRegistry<M, T>,
    ) -> Result<Self, DuplicateComponentError>
    where
        B: NewBundle,
        M: AsRef<FieldDescriptor> + WithErasedDrop + FromComponentType,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let archetype = ErasedArchetype::register::<B, M, T>(components)?;
        let me = Self::from_archetype(archetype);
        Ok(me)
    }

    #[inline]
    pub fn from_archetype(archetype: ErasedArchetype<ErasedDropMeta>) -> Self {
        let sparse_set = EpochSparseSet::with_context(archetype);
        Self { sparse_set }
    }

    #[inline]
    pub fn archetype(&self) -> &ErasedArchetype<ErasedDropMeta> {
        let Self { sparse_set } = self;
        sparse_set.context()
    }

    #[inline]
    pub fn into_archetype(self) -> ErasedArchetype<ErasedDropMeta> {
        let Self { sparse_set } = self;

        let (dense, _) = sparse_set.into_parts();
        dense.into_context().into_inner()
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
    pub fn bundles<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(&[Entity], BundleSlices<'_, B>), IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (entities, bundles) = self.as_slices();
        let bundles = bundles.downcast::<B, T>(components)?;
        Ok((entities, bundles))
    }

    #[inline]
    pub fn bundles_mut<B, T>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(&[Entity], BundleSlicesMut<'_, B>), IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let (entities, bundles) = self.as_mut_slices();
        let bundles = bundles.downcast::<B, T>(components)?;
        Ok((entities, bundles))
    }

    #[inline]
    pub fn get_bundle<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Some(bundle) = self.get(entity) else {
            return Ok(None);
        };

        let bundle = bundle.downcast::<B, T>(components)?;
        Ok(Some(bundle))
    }

    #[inline]
    pub fn get_bundle_mut<B, T>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
    ) -> Result<Option<BundleRefsMut<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Some(bundle) = self.get_mut(entity) else {
            return Ok(None);
        };

        let bundle = bundle.downcast::<B, T>(components)?;
        Ok(Some(bundle))
    }

    #[inline]
    pub fn insert_bundle<B, T>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
        value: B,
    ) -> Result<Option<B>, IncompatibleBundleValueError<B>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        if let Err(reason) = self
            .archetype()
            .check_exact_compatibility_of::<B, T>(components)
        {
            let error = IncompatibleBundleValueError { value, reason };
            return Err(error);
        }

        let Self { sparse_set } = self;
        let bundle = sparse_set.insert_from(entity.into(), |_, access| match access? {
            TryInsertAccess::ReadWrite(dst) => {
                let dst = dst
                    .into_ptrs()
                    .downcast::<B, T>(components)
                    .expect("exact archetype compatibility should be already checked");
                let value = unsafe { soa::ptr::replace::<B, _, _>(B::CONTEXT, dst, value) };
                Some(value)
            }
            TryInsertAccess::WriteOnly(dst) => {
                let dst = dst
                    .into_inner()
                    .downcast::<B, T>(components)
                    .expect("exact archetype compatibility should be already checked");
                unsafe { B::CONTEXT.write(dst, value) };
                None
            }
        });
        Ok(bundle)
    }

    #[inline]
    pub fn remove_bundle<B, T>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
    ) -> Result<Option<B>, IncompatibleArchetypeExactError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.archetype()
            .check_exact_compatibility_of::<B, T>(components)?;

        let Self { sparse_set } = self;
        let bundle = sparse_set.swap_remove_into(entity.into(), |_, src| {
            let src = src?
                .cast_const()
                .downcast::<B, T>(components)
                .expect("exact archetype compatibility should be already checked");
            let bundle = unsafe { B::CONTEXT.read(src) };
            Some(bundle)
        });
        Ok(bundle)
    }

    #[inline]
    pub fn get(
        &self,
        entity: Entity,
    ) -> Option<ErasedBundleRefs<'_, &ErasedArchetype<ErasedDropMeta>>> {
        let Self { sparse_set } = self;
        sparse_set.get(entity.into())
    }

    #[inline]
    pub fn get_mut(
        &mut self,
        entity: Entity,
    ) -> Option<ErasedBundleMutRefs<'_, &ErasedArchetype<ErasedDropMeta>>> {
        let Self { sparse_set } = self;
        sparse_set.get_mut(entity.into())
    }

    #[inline]
    pub fn insert<T>(
        &mut self,
        entity: Entity,
        bundle: ErasedBundleKind<T>,
    ) -> Result<Option<ErasedBorrowedBundle<'_, ErasedDropMeta>>, IncompatibleArchetypeExactError>
    where
        T: ErasedArchetypeKind<Meta = ErasedDropMeta>,
    {
        let archetype = self.archetype();
        archetype.check_exact_compatibility(bundle.archetype().as_view())?;

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
    pub fn remove(&mut self, entity: Entity) -> Option<ErasedBorrowedBundle<'_, ErasedDropMeta>> {
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
    pub fn destroy_all(&mut self) {
        let Self { sparse_set } = self;
        sparse_set.clear_sparse();
    }

    #[inline]
    pub fn as_slices(
        &self,
    ) -> (
        &[Entity],
        ErasedBundleSlices<'_, &ErasedArchetype<ErasedDropMeta>>,
    ) {
        let Self { sparse_set } = self;

        let (dense, _) = sparse_set.as_view().into_parts();
        let (entities, slices) = dense.into_slices().into_parts();

        let entities = must_cast_slice(entities);
        (entities, slices)
    }

    #[inline]
    pub fn as_mut_slices(
        &mut self,
    ) -> (
        &[Entity],
        ErasedBundleMutSlices<'_, &ErasedArchetype<ErasedDropMeta>>,
    ) {
        let Self { sparse_set } = self;

        let (dense, _) = sparse_set.as_mut_view().into_parts();
        let (entities, slices) = dense.into_slices().into_parts();

        let entities = must_cast_slice(entities);
        (entities, slices)
    }
}

impl Debug for ArchetypeStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let component_ids = &self.archetype().component_ids();
        f.debug_struct("ArchetypeStorage")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}
