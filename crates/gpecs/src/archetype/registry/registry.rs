#![expect(clippy::module_inception)]

use std::fmt::{self, Debug};

use crate::{
    archetype::{
        ErasedDropMeta,
        erased::{
            ErasedArchetype, ErasedArchetypeView,
            error::{ArchetypeError, DuplicateComponentError, MissingComponentError},
        },
        registry::{
            ArchetypeId, ArchetypeIds, ArchetypeInfo, ArchetypesAfter, ArchetypesAfterMut,
            ArchetypesBefore, ArchetypesBeforeMut, Bundles, BundlesMut, CompatibleArchetypes,
            CompatibleArchetypesMut, EntityLocation, ErasedArchetypeCow, Iter, IterMut,
            error::{
                GetAtError, InsertAtError, InsertBundleAtError, InsertBundleError,
                InsertBundleExactAtError, InsertBundleExactError, InsertExactAtError,
                InsertExactError, InvalidEntityLocationError, RemoveBundleAtError,
                RemoveBundleExactAtError, RemoveBundleExactError, RemoveExactAtError,
            },
        },
        storage::{
            ArchetypeStorage,
            error::{MoveIntoWithInsertBundleError, UpdateWithBundleError},
        },
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut,
        erased::{
            ErasedBorrowedViewBundle, ErasedBundle, ErasedBundleKind, ErasedBundleMutRefs,
            ErasedBundleRefs, RemovePair,
            error::{DowncastError, DowncastErrorKind, FromBundleError},
            traits::ErasedArchetypeKind,
        },
    },
    component::{
        erased::WithErasedDrop,
        registry::{
            ComponentId, ComponentRegistry, ComponentRegistryView,
            traits::{
                ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType, PushBackArray,
            },
        },
    },
    entity::Entity,
    soa::layout::WithLayout,
};

use super::algo;

#[derive(Default)]
pub struct ArchetypeRegistry {
    archetypes: algo::Archetypes,
    graph: algo::Graph,
}

impl ArchetypeRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            archetypes: algo::Archetypes::default(),
            graph: algo::Graph::default(),
        }
    }

    #[inline]
    pub fn register_archetype_of<B, M, T>(
        &mut self,
        components: &mut ComponentRegistry<M, T>,
    ) -> Result<ArchetypeId, DuplicateComponentError>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let archetype = ErasedArchetype::register::<B, _, _>(components)?;
        let archetype_id = self.register_archetype(&components.as_view(), archetype);
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_archetype_from<I, M>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        component_ids: I,
    ) -> Result<ArchetypeId, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
        M: WithLayout + WithErasedDrop,
    {
        let archetype = ErasedArchetype::new(components, component_ids)?;
        let archetype_id = self.register_archetype(components, archetype);
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_archetype<'a, M>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        archetype: impl Into<ErasedArchetypeCow<'a, ErasedDropMeta>>,
    ) -> ArchetypeId
    where
        M: WithLayout + WithErasedDrop,
    {
        let Self { archetypes, graph } = self;
        algo::register(archetypes, graph, components, archetype.into())
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { archetypes, .. } = self;
        archetypes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { archetypes, .. } = self;
        archetypes.is_empty()
    }

    #[inline]
    pub fn get_archetype_info(&self, id: ArchetypeId) -> Option<ArchetypeInfo<&ArchetypeStorage>> {
        let Self { archetypes, .. } = self;

        let storage = algo::get_archetype_storage(archetypes, id)?;
        let info = ArchetypeInfo::new(id, storage);
        Some(info)
    }

    #[inline]
    pub unsafe fn get_archetype_info_mut(
        &mut self,
        id: ArchetypeId,
    ) -> Option<ArchetypeInfo<&mut ArchetypeStorage>> {
        let Self { archetypes, .. } = self;

        let storage = algo::get_archetype_storage_mut(archetypes, id)?;
        let info = ArchetypeInfo::new(id, storage);
        Some(info)
    }

    #[inline]
    pub fn archetype_id_from<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<Option<ArchetypeId>, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = ErasedArchetype::<()>::new(components, component_ids)?;
        let archetype_id = self.archetype_id(archetype.as_view());
        Ok(archetype_id)
    }

    #[inline]
    pub fn archetype_id_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<Option<ArchetypeId>, ArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let archetype = ErasedArchetype::<()>::of::<B, _, T>(components)?;
        let archetype_id = self.archetype_id(archetype.as_view());
        Ok(archetype_id)
    }

    #[inline]
    pub fn archetype_id(&self, archetype: ErasedArchetypeView<impl Sized>) -> Option<ArchetypeId> {
        let Self { archetypes, .. } = self;
        algo::find_archetype(archetypes, archetype)
    }

    #[inline]
    pub fn archetype_ids(&self) -> ArchetypeIds {
        let Self { archetypes, .. } = self;
        ArchetypeIds::new(archetypes)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        let Self { archetypes, .. } = self;
        Iter::new(archetypes)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        let Self { archetypes, .. } = self;
        IterMut::new(archetypes)
    }

    #[inline]
    pub fn archetypes_before(&self, id: ArchetypeId) -> Option<ArchetypesBefore<'_>> {
        let Self { archetypes, graph } = self;
        ArchetypesBefore::new(archetypes, graph, id, true)
    }

    #[inline]
    pub unsafe fn archetypes_before_mut(
        &mut self,
        id: ArchetypeId,
    ) -> Option<ArchetypesBeforeMut<'_>> {
        let Self { archetypes, graph } = self;
        ArchetypesBeforeMut::new(archetypes, graph, id, true)
    }

    #[inline]
    pub fn archetypes_before_inclusive(&self, id: ArchetypeId) -> Option<ArchetypesBefore<'_>> {
        let Self { archetypes, graph } = self;
        ArchetypesBefore::new(archetypes, graph, id, false)
    }

    #[inline]
    pub unsafe fn archetypes_before_inclusive_mut(
        &mut self,
        id: ArchetypeId,
    ) -> Option<ArchetypesBeforeMut<'_>> {
        let Self { archetypes, graph } = self;
        ArchetypesBeforeMut::new(archetypes, graph, id, false)
    }

    #[inline]
    pub fn archetypes_after(&self, id: ArchetypeId) -> Option<ArchetypesAfter<'_>> {
        let Self { archetypes, graph } = self;
        ArchetypesAfter::new(archetypes, graph, id, true)
    }

    #[inline]
    pub unsafe fn archetypes_after_mut(
        &mut self,
        id: ArchetypeId,
    ) -> Option<ArchetypesAfterMut<'_>> {
        let Self { archetypes, graph } = self;
        ArchetypesAfterMut::new(archetypes, graph, id, true)
    }

    #[inline]
    pub fn archetypes_after_inclusive(&self, id: ArchetypeId) -> Option<ArchetypesAfter<'_>> {
        let Self { archetypes, graph } = self;
        ArchetypesAfter::new(archetypes, graph, id, false)
    }

    #[inline]
    pub unsafe fn archetypes_after_inclusive_mut(
        &mut self,
        id: ArchetypeId,
    ) -> Option<ArchetypesAfterMut<'_>> {
        let Self { archetypes, graph } = self;
        ArchetypesAfterMut::new(archetypes, graph, id, false)
    }

    #[inline]
    pub fn compatible_archetypes_from<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<CompatibleArchetypes<'_>, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = ErasedArchetype::<()>::new(components, component_ids)?;
        let archetypes = self.compatible_archetypes(archetype.as_view());
        Ok(archetypes)
    }

    #[inline]
    pub fn compatible_archetypes_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<CompatibleArchetypes<'_>, ArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let archetype = ErasedArchetype::<()>::of::<B, _, T>(components)?;
        let archetypes = self.compatible_archetypes(archetype.as_view());
        Ok(archetypes)
    }

    #[inline]
    pub fn compatible_archetypes(
        &self,
        archetype: ErasedArchetypeView<impl Sized>,
    ) -> CompatibleArchetypes<'_> {
        let Self { archetypes, graph } = self;
        CompatibleArchetypes::new(archetypes, graph, archetype)
    }

    #[inline]
    pub unsafe fn compatible_archetypes_mut_from<I>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<CompatibleArchetypesMut<'_>, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = ErasedArchetype::<()>::new(components, component_ids)?;
        let archetypes = self.compatible_archetypes_mut(archetype.as_view());
        Ok(archetypes)
    }

    #[inline]
    pub unsafe fn compatible_archetypes_mut_of<B, T>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<CompatibleArchetypesMut<'_>, ArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let archetype = ErasedArchetype::<()>::of::<B, _, T>(components)?;
        let archetypes = self.compatible_archetypes_mut(archetype.as_view());
        Ok(archetypes)
    }

    #[inline]
    pub fn compatible_archetypes_mut(
        &mut self,
        archetype: ErasedArchetypeView<impl Sized>,
    ) -> CompatibleArchetypesMut<'_> {
        let Self { archetypes, graph } = self;
        CompatibleArchetypesMut::new(archetypes, graph, archetype)
    }

    #[inline]
    pub fn find_location(&self, entity: Entity) -> EntityLocation {
        let Self { archetypes, .. } = self;
        algo::find_location(archetypes, entity)
    }

    #[inline]
    pub fn check_location(
        &self,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<(), InvalidEntityLocationError> {
        let Self { archetypes, .. } = self;
        algo::check_location(archetypes, entity, location)
    }

    #[inline]
    pub fn get(
        &self,
        entity: Entity,
    ) -> Option<ErasedBundleRefs<'_, &ErasedArchetype<ErasedDropMeta>>> {
        let location = self.find_location(entity);
        let Ok(bundle) = self
            .get_at(entity, location)
            .map_err(InvalidEntityLocationError::with_valid_location);
        bundle
    }

    #[inline]
    pub fn get_at(
        &self,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<
        Option<ErasedBundleRefs<'_, &ErasedArchetype<ErasedDropMeta>>>,
        InvalidEntityLocationError,
    > {
        self.check_location(entity, location)?;
        let EntityLocation::WithComponents(archetype_id) = location else {
            return Ok(None);
        };

        let Self { archetypes, .. } = self;

        let storage = algo::unwrap_archetype_storage(archetypes, archetype_id);
        let bundle = storage.get(entity);
        Ok(bundle)
    }

    #[inline]
    pub fn get_bundle<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'_, B>>, DowncastErrorKind>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let location = self.find_location(entity);
        self.get_bundle_at::<B, T>(components, entity, location)
            .map_err(GetAtError::with_valid_location)
    }

    #[inline]
    pub fn get_bundle_at<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<Option<BundleRefs<'_, B>>, GetAtError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Some(bundle) = self.get_at(entity, location)? else {
            return Ok(None);
        };

        let bundle = bundle
            .downcast::<B, T>(components)
            .map_err(DowncastError::into_source)?;
        Ok(Some(bundle))
    }

    #[inline]
    pub fn get_mut(
        &mut self,
        entity: Entity,
    ) -> Option<ErasedBundleMutRefs<'_, &ErasedArchetype<ErasedDropMeta>>> {
        let location = self.find_location(entity);
        let Ok(bundle) = self
            .get_mut_at(entity, location)
            .map_err(InvalidEntityLocationError::with_valid_location);
        bundle
    }

    #[inline]
    pub fn get_mut_at(
        &mut self,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<
        Option<ErasedBundleMutRefs<'_, &ErasedArchetype<ErasedDropMeta>>>,
        InvalidEntityLocationError,
    > {
        self.check_location(entity, location)?;
        let EntityLocation::WithComponents(archetype_id) = location else {
            return Ok(None);
        };

        let Self { archetypes, .. } = self;

        let storage = algo::unwrap_archetype_storage_mut(archetypes, archetype_id);
        let bundle = storage.get_mut(entity);
        Ok(bundle)
    }

    #[inline]
    pub fn get_bundle_mut<B, T>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
    ) -> Result<Option<BundleRefsMut<'_, B>>, DowncastErrorKind>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let location = self.find_location(entity);
        self.get_bundle_mut_at::<B, T>(components, entity, location)
            .map_err(GetAtError::with_valid_location)
    }

    #[inline]
    pub fn get_bundle_mut_at<B, T>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, T>,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<Option<BundleRefsMut<'_, B>>, GetAtError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Some(bundle) = self.get_mut_at(entity, location)? else {
            return Ok(None);
        };

        let bundle = bundle
            .downcast::<B, T>(components)
            .map_err(DowncastError::into_source)?;
        Ok(Some(bundle))
    }

    #[inline]
    pub fn bundles<'ctx, B, M, T>(
        &self,
        components: ComponentRegistryView<'ctx, M, T>,
    ) -> Result<Bundles<'_, 'ctx, B, M, T>, ArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType>,
    {
        let Self { archetypes, graph } = self;
        Bundles::new(archetypes, graph, components)
    }

    #[inline]
    pub fn bundles_mut<'ctx, B, M, T>(
        &mut self,
        components: ComponentRegistryView<'ctx, M, T>,
    ) -> Result<BundlesMut<'_, 'ctx, B, M, T>, ArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType>,
    {
        let Self { archetypes, graph } = self;
        BundlesMut::new(archetypes, graph, components)
    }

    #[inline]
    pub fn insert_exact<T, M>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        entity: Entity,
        value: ErasedBundleKind<T>,
    ) -> Result<(), InsertExactError<ErasedBundleKind<T>>>
    where
        T: ErasedArchetypeKind<Meta = ErasedDropMeta>,
        M: WithLayout + WithErasedDrop,
    {
        let location = self.find_location(entity);
        self.insert_exact_at(components, entity, value, location)
            .map_err(InsertExactAtError::with_valid_location)?;
        Ok(())
    }

    #[inline]
    pub fn insert_exact_at<T, M>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        entity: Entity,
        value: ErasedBundleKind<T>,
        location: EntityLocation,
    ) -> Result<ArchetypeId, InsertExactAtError<ErasedBundleKind<T>>>
    where
        T: ErasedArchetypeKind<Meta = ErasedDropMeta>,
        M: WithLayout + WithErasedDrop,
    {
        let Self { archetypes, graph } = self;
        let result = algo::insert_exact_archetypes(
            graph,
            archetypes,
            components,
            entity,
            location,
            value.archetype(),
        );
        let (old_archetype, new_archetype) = match result {
            Ok(archetypes) => archetypes,
            Err(source) => return Err(InsertExactAtError { value, source }),
        };

        let Some(old_archetype) = old_archetype else {
            algo::insert_into_archetype(archetypes, new_archetype, entity, value);
            return Ok(new_archetype);
        };

        let (old_storage, new_storage) =
            algo::unwrap_archetype_storage_pair_mut(archetypes, old_archetype, new_archetype);
        old_storage
            .move_into_with_insert(new_storage, entity, value)
            .expect("bundle should be moved successfully");

        Ok(new_archetype)
    }

    #[inline]
    pub fn insert_bundle_exact<B, M, T>(
        &mut self,
        components: &mut ComponentRegistry<M, T>,
        entity: Entity,
        value: B,
    ) -> Result<(), InsertBundleExactError<B>>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let location = self.find_location(entity);
        self.insert_bundle_exact_at::<B, M, T>(components, entity, value, location)
            .map_err(InsertBundleExactAtError::with_valid_location)?;
        Ok(())
    }

    #[inline]
    pub fn insert_bundle_exact_at<B, M, T>(
        &mut self,
        components: &mut ComponentRegistry<M, T>,
        entity: Entity,
        value: B,
        location: EntityLocation,
    ) -> Result<ArchetypeId, InsertBundleExactAtError<B>>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let components_to_insert = match ErasedArchetype::register::<B, M, T>(components) {
            Ok(archetype) => archetype,
            Err(error) => {
                let source = error.into();
                return Err(InsertBundleExactAtError { value, source });
            }
        };

        let Self { archetypes, graph } = self;
        let components_view = &components.as_view();
        let result = algo::insert_exact_archetypes(
            graph,
            archetypes,
            components_view,
            entity,
            location,
            components_to_insert.as_view(),
        );
        let (old_archetype, new_archetype) = match result {
            Ok(archetypes) => archetypes,
            Err(error) => {
                let source = error.into();
                return Err(InsertBundleExactAtError { value, source });
            }
        };

        let Some(old_archetype) = old_archetype else {
            let id = new_archetype;
            algo::insert_bundle_into_archetype(archetypes, components_view, id, entity, value);
            return Ok(new_archetype);
        };

        let (old_storage, new_storage) =
            algo::unwrap_archetype_storage_pair_mut(archetypes, old_archetype, new_archetype);
        old_storage
            .move_into_with_insert_bundle(components_view, new_storage, entity, value)
            .map_err(MoveIntoWithInsertBundleError::into_source)
            .expect("bundle should be moved successfully");

        Ok(new_archetype)
    }

    #[inline]
    pub fn insert<T, M>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        entity: Entity,
        value: ErasedBundleKind<T>,
    ) where
        T: ErasedArchetypeKind<Meta = ErasedDropMeta>,
        M: WithLayout + WithErasedDrop,
    {
        let location = self.find_location(entity);
        let Ok(_) = self
            .insert_at(components, entity, value, location)
            .map_err(InsertAtError::with_valid_location);
    }

    #[inline]
    pub fn insert_at<T, M>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        entity: Entity,
        value: ErasedBundleKind<T>,
        location: EntityLocation,
    ) -> Result<ArchetypeId, InsertAtError<ErasedBundleKind<T>>>
    where
        T: ErasedArchetypeKind<Meta = ErasedDropMeta>,
        M: WithLayout + WithErasedDrop,
    {
        let Self { archetypes, graph } = self;
        let result = algo::insert_archetypes(
            graph,
            archetypes,
            components,
            entity,
            location,
            value.archetype(),
        );
        let (old_archetype, new_archetype) = match result {
            Ok(archetypes) => archetypes,
            Err(source) => return Err(InsertAtError { value, source }),
        };

        let Some(old_archetype) = old_archetype else {
            algo::insert_into_archetype(archetypes, new_archetype, entity, value);
            return Ok(new_archetype);
        };

        if let Some((_old_storage, _new_storage)) =
            algo::get_archetype_storage_pair_mut(archetypes, old_archetype, new_archetype)
        {
            // FIXME: update existing components & move new ones into new archetype
            let bundle = algo::remove_from_archetype(archetypes, old_archetype, entity)
                .replace(value)
                .expect("combined bundle should be created successfully");
            algo::insert_into_archetype(archetypes, new_archetype, entity, bundle);
        } else {
            assert_eq!(old_archetype, new_archetype);
            let storage = algo::unwrap_archetype_storage_mut(archetypes, new_archetype);
            storage
                .update_with(entity, value)
                .expect("entity should exist in storage & all value components should be present");
        }

        Ok(new_archetype)
    }

    #[inline]
    pub fn insert_bundle<B, M, T>(
        &mut self,
        components: &mut ComponentRegistry<M, T>,
        entity: Entity,
        value: B,
    ) -> Result<(), InsertBundleError<B>>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let location = self.find_location(entity);
        self.insert_bundle_at::<B, M, T>(components, entity, value, location)
            .map_err(InsertBundleAtError::into_insert_bundle_error)?;
        Ok(())
    }

    #[inline]
    pub fn insert_bundle_at<B, M, T>(
        &mut self,
        components: &mut ComponentRegistry<M, T>,
        entity: Entity,
        value: B,
        location: EntityLocation,
    ) -> Result<ArchetypeId, InsertBundleAtError<B>>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let components_to_insert = match ErasedArchetype::register::<B, M, T>(components) {
            Ok(archetype) => archetype,
            Err(error) => {
                let source = error.into();
                return Err(InsertBundleAtError { value, source });
            }
        };

        let Self { archetypes, graph } = self;
        let components_view = &components.as_view();
        let result = algo::insert_archetypes(
            graph,
            archetypes,
            components_view,
            entity,
            location,
            components_to_insert.as_view(),
        );
        let (old_archetype, new_archetype) = match result {
            Ok(archetypes) => archetypes,
            Err(error) => {
                let source = error.into();
                return Err(InsertBundleAtError { value, source });
            }
        };

        let Some(old_archetype) = old_archetype else {
            let id = new_archetype;
            algo::insert_bundle_into_archetype(archetypes, components_view, id, entity, value);
            return Ok(new_archetype);
        };

        if let Some((_old_storage, _new_storage)) =
            algo::get_archetype_storage_pair_mut(archetypes, old_archetype, new_archetype)
        {
            // FIXME: update existing components & move new ones into new archetype
            let to_replace = ErasedBundle::from_bundle(components, value)
                .map_err(FromBundleError::into_source)
                .expect("bundle compatibility should have been already checked");
            let bundle = algo::remove_from_archetype(archetypes, old_archetype, entity)
                .replace(to_replace)
                .expect("combined bundle should be created successfully");
            algo::insert_into_archetype(archetypes, new_archetype, entity, bundle);
        } else {
            assert_eq!(old_archetype, new_archetype);
            let storage = algo::unwrap_archetype_storage_mut(archetypes, new_archetype);
            storage
                .update_with_bundle(components_view, entity, value)
                .map_err(UpdateWithBundleError::into_source)
                .expect("entity should exist in storage & bundle compatibility should have been already checked");
        }

        Ok(new_archetype)
    }

    #[inline]
    pub fn remove_exact<'me, M>(
        &'me mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        entity: Entity,
        components_to_remove: ErasedArchetypeView<'me, ErasedDropMeta>,
    ) -> Result<Option<ErasedBorrowedViewBundle<'me, ErasedDropMeta>>, MissingComponentError>
    where
        M: WithLayout + WithErasedDrop,
    {
        let location = self.find_location(entity);
        let (value, _) = self
            .remove_exact_at(components, entity, components_to_remove, location)
            .map_err(RemoveExactAtError::with_valid_location)?;
        Ok(value)
    }

    #[inline]
    pub fn remove_exact_at<'me, M>(
        &'me mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        entity: Entity,
        components_to_remove: ErasedArchetypeView<'me, ErasedDropMeta>,
        location: EntityLocation,
    ) -> Result<
        (
            Option<ErasedBorrowedViewBundle<'me, ErasedDropMeta>>,
            EntityLocation,
        ),
        RemoveExactAtError,
    >
    where
        M: WithLayout + WithErasedDrop,
    {
        let Self { archetypes, graph } = self;
        let remove_archetypes = algo::remove_exact_archetypes(
            graph,
            archetypes,
            components,
            entity,
            location,
            components_to_remove,
        )?;
        let Some((old_archetype, new_archetype)) = remove_archetypes else {
            return Ok((None, EntityLocation::WithoutComponents));
        };

        let Some(new_archetype) = new_archetype else {
            let value = algo::remove_from_archetype(archetypes, old_archetype, entity);
            return Ok((Some(value.into()), EntityLocation::WithoutComponents));
        };

        // FIXME: can we optimize this (by writing into a new archetype directly)?
        let (_old_storage, _new_storage) =
            algo::unwrap_archetype_storage_pair_mut(archetypes, old_archetype, new_archetype);

        let RemovePair {
            retained: bundle,
            removed: value,
        } = algo::remove_from_archetype(archetypes, old_archetype, entity)
            .remove(components_to_remove)
            .expect("all the bundle components should be present in the old archetype");
        algo::insert_into_archetype(archetypes, new_archetype, entity, bundle);

        let location = EntityLocation::WithComponents(new_archetype);
        Ok((Some(value), location))
    }

    #[inline]
    pub fn remove_bundle_exact<B, M, T>(
        &mut self,
        components: &mut ComponentRegistry<M, T>,
        entity: Entity,
    ) -> Result<Option<B>, RemoveBundleExactError>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let location = self.find_location(entity);
        let (value, _) = self
            .remove_bundle_exact_at::<B, M, T>(components, entity, location)
            .map_err(RemoveBundleExactAtError::with_valid_location)?;
        Ok(value)
    }

    #[inline]
    pub fn remove_bundle_exact_at<B, M, T>(
        &mut self,
        components: &mut ComponentRegistry<M, T>,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<(Option<B>, EntityLocation), RemoveBundleExactAtError>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let components_to_remove = ErasedArchetype::register::<B, M, T>(components)?;

        let Self { archetypes, graph } = self;
        let components = &components.as_view();
        let remove_archetypes = algo::remove_exact_archetypes(
            graph,
            archetypes,
            components,
            entity,
            location,
            components_to_remove.as_view(),
        )?;
        let Some((old_archetype, new_archetype)) = remove_archetypes else {
            return Ok((None, EntityLocation::WithoutComponents));
        };

        let Some(new_archetype) = new_archetype else {
            let id = old_archetype;
            let value = algo::remove_bundle_from_archetype(archetypes, components, id, entity);
            return Ok((Some(value), EntityLocation::WithoutComponents));
        };

        // FIXME: can we optimize this (by writing into a new archetype directly)?
        let (_old_storage, _new_storage) =
            algo::unwrap_archetype_storage_pair_mut(archetypes, old_archetype, new_archetype);

        let RemovePair {
            retained: bundle,
            removed,
        } = algo::remove_from_archetype(archetypes, old_archetype, entity)
            .remove(components_to_remove)
            .expect("all the bundle components should be present in the old archetype");
        algo::insert_into_archetype(archetypes, new_archetype, entity, bundle);

        let value = removed
            .downcast(components)
            .expect("archetype should be compatible");
        let location = EntityLocation::WithComponents(new_archetype);
        Ok((Some(value), location))
    }

    #[inline]
    pub fn remove<M>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        entity: Entity,
        components_to_remove: ErasedArchetypeView<impl Sized>,
    ) where
        M: WithLayout + WithErasedDrop,
    {
        let location = self.find_location(entity);
        let Ok(_) = self
            .remove_at(components, entity, components_to_remove, location)
            .map_err(InvalidEntityLocationError::with_valid_location);
    }

    #[inline]
    pub fn remove_at<M>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
        entity: Entity,
        components_to_remove: ErasedArchetypeView<impl Sized>,
        location: EntityLocation,
    ) -> Result<EntityLocation, InvalidEntityLocationError>
    where
        M: WithLayout + WithErasedDrop,
    {
        let Self { archetypes, graph } = self;
        let remove_archetypes = algo::remove_archetypes(
            graph,
            archetypes,
            components,
            entity,
            location,
            components_to_remove,
        )?;
        let Some((old_archetype, new_archetype)) = remove_archetypes else {
            return Ok(EntityLocation::WithoutComponents);
        };

        let Some(new_archetype) = new_archetype else {
            algo::destroy_in_archetype(archetypes, old_archetype, entity);
            return Ok(EntityLocation::WithoutComponents);
        };

        // FIXME: can we optimize this (by writing into a new archetype directly)?
        let (_old_storage, _new_storage) =
            algo::unwrap_archetype_storage_pair_mut(archetypes, old_archetype, new_archetype);

        let bundle = algo::remove_from_archetype(archetypes, old_archetype, entity)
            .destroy(components_to_remove)
            .expect("all the bundle components should be present in the old archetype");
        algo::insert_into_archetype(archetypes, new_archetype, entity, bundle);

        let location = EntityLocation::WithComponents(new_archetype);
        Ok(location)
    }

    #[inline]
    pub fn remove_bundle<B, M, T>(
        &mut self,
        components: &mut ComponentRegistry<M, T>,
        entity: Entity,
    ) -> Result<(), DuplicateComponentError>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let location = self.find_location(entity);
        self.remove_bundle_at::<B, M, T>(components, entity, location)
            .map_err(RemoveBundleAtError::with_valid_location)?;
        Ok(())
    }

    #[inline]
    pub fn remove_bundle_at<B, M, T>(
        &mut self,
        components: &mut ComponentRegistry<M, T>,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<EntityLocation, RemoveBundleAtError>
    where
        B: Bundle,
        M: PushBackArray<Item: WithLayout + WithErasedDrop + FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let components_to_remove = ErasedArchetype::<()>::register::<B, M, T>(components)?;
        let components_to_remove = components_to_remove.as_view();

        let Self { archetypes, graph } = self;
        let components = &components.as_view();
        let remove_archetypes = algo::remove_archetypes(
            graph,
            archetypes,
            components,
            entity,
            location,
            components_to_remove,
        )?;
        let Some((old_archetype, new_archetype)) = remove_archetypes else {
            return Ok(EntityLocation::WithoutComponents);
        };

        let Some(new_archetype) = new_archetype else {
            algo::destroy_in_archetype(archetypes, old_archetype, entity);
            return Ok(EntityLocation::WithoutComponents);
        };

        // FIXME: can we optimize this (by writing into a new archetype directly)?
        let (_old_storage, _new_storage) =
            algo::unwrap_archetype_storage_pair_mut(archetypes, old_archetype, new_archetype);

        let bundle = algo::remove_from_archetype(archetypes, old_archetype, entity)
            .destroy(components_to_remove)
            .expect("all the bundle components should be present in the old archetype");
        algo::insert_into_archetype(archetypes, new_archetype, entity, bundle);

        let location = EntityLocation::WithComponents(new_archetype);
        Ok(location)
    }

    #[inline]
    pub fn destroy(&mut self, entity: Entity) -> bool {
        let location = self.find_location(entity);
        let Ok(destroyed) = self
            .destroy_at(entity, location)
            .map_err(InvalidEntityLocationError::with_valid_location);
        destroyed
    }

    #[inline]
    pub fn destroy_at(
        &mut self,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<bool, InvalidEntityLocationError> {
        self.check_location(entity, location)?;
        let EntityLocation::WithComponents(archetype_id) = location else {
            return Ok(false);
        };

        let Self { archetypes, .. } = self;
        algo::destroy_in_archetype(archetypes, archetype_id, entity);
        Ok(true)
    }

    #[inline]
    pub fn destroy_all(&mut self) {
        let archetype_ids = self.archetype_ids();
        let Self { archetypes, .. } = self;

        for archetype_id in archetype_ids {
            let storage = algo::unwrap_archetype_storage_mut(archetypes, archetype_id);
            storage.destroy_all();
        }
    }
}

impl Debug for ArchetypeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, graph } = self;

        algo::graph_dot_scoped(archetypes, graph, |graph| {
            f.debug_struct("ArchetypeRegistry")
                .field("archetypes", archetypes)
                .field("graph", graph)
                .finish()
        })
    }
}

impl<'a> IntoIterator for &'a ArchetypeRegistry {
    type Item = ArchetypeInfo<&'a ArchetypeStorage>;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut ArchetypeRegistry {
    type Item = ArchetypeInfo<&'a mut ArchetypeStorage>;
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
