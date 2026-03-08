use std::{
    borrow::Borrow,
    collections::BTreeSet,
    fmt::{self, Debug},
    iter::{self, FusedIterator},
    marker::PhantomData,
    ops::Range,
    slice,
};

pub use gpecs_types::archetype::ArchetypeId;

use indexmap::map::{Values as IndexMapValues, ValuesMut as IndexMapValuesMut};
use itertools::Itertools;
use petgraph::{
    Direction,
    dot::{Config as DotConfig, Dot, RankDir},
    graph::{DiGraph, EdgeReference, NodeIndex},
    visit::{Bfs, EdgeRef, Reversed, Visitable, Walker, WalkerIter},
};

use crate::{
    archetype::{
        collect::{try_collect_components, try_collect_opt_components},
        error::{
            AlreadyHasComponentError, ArchetypeError, DuplicateComponentError,
            IncompatibleArchetypeError, InsertBundleError, InsertBundleExactError,
            MissingComponentError, RemoveBundleExactError,
        },
        storage::{ArchetypeStorage, StorageMeta},
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut,
        erased::{ErasedArchetypeKind, ErasedBorrowedBundle, ErasedBundle, ErasedBundleKind},
    },
    component::registry::{ComponentId, ComponentRegistry},
    entity::Entity,
    hash::{IndexMap, IndexSet},
    soa::slice::{Iter as SoaIter, IterMut as SoaIterMut, SoaSlices, SoaSlicesMut},
};

/// Archetype [identifier](ArchetypeId) of some [entity](Entity).
///
/// [`None`] means that an entity has no components attached to it.
pub type EntityArchetype = Option<ArchetypeId>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum EntityArchetypeLocation {
    Unknown,
    Known(EntityArchetype),
}

impl EntityArchetypeLocation {
    #[inline]
    pub fn from_option(option: Option<EntityArchetype>) -> Self {
        match option {
            Some(archetype_id) => Self::Known(archetype_id),
            None => Self::Unknown,
        }
    }

    #[inline]
    pub fn into_option(self) -> Option<EntityArchetype> {
        match self {
            Self::Unknown => None,
            Self::Known(archetype_id) => Some(archetype_id),
        }
    }
}

impl From<EntityArchetype> for EntityArchetypeLocation {
    #[inline]
    fn from(value: EntityArchetype) -> Self {
        Self::Known(value)
    }
}

impl From<Option<EntityArchetype>> for EntityArchetypeLocation {
    #[inline]
    fn from(value: Option<EntityArchetype>) -> Self {
        Self::from_option(value)
    }
}

impl From<EntityArchetypeLocation> for Option<EntityArchetype> {
    #[inline]
    fn from(value: EntityArchetypeLocation) -> Self {
        EntityArchetypeLocation::into_option(value)
    }
}

#[derive(Debug)]
pub struct ArchetypeInfo {
    id: ArchetypeId,
    storage: ArchetypeStorage,
}

impl ArchetypeInfo {
    #[inline]
    pub fn id(&self) -> ArchetypeId {
        let Self { id, .. } = *self;
        id
    }

    #[inline]
    pub fn storage(&self) -> &ArchetypeStorage {
        let Self { storage, .. } = self;
        storage
    }

    #[inline]
    pub unsafe fn storage_mut(&mut self) -> &mut ArchetypeStorage {
        let Self { storage, .. } = self;
        storage
    }
}

type ArchetypeKey = BTreeSet<ComponentId>;
type Archetypes = IndexMap<ArchetypeKey, ArchetypeInfo>;
type Graph = DiGraph<(), ComponentId, u32>;

#[derive(Default)]
pub struct ArchetypeRegistry {
    archetypes: Archetypes,
    graph: Graph,
}

impl ArchetypeRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            archetypes: Archetypes::default(),
            graph: Graph::default(),
        }
    }

    #[inline]
    pub fn register_archetype<B>(
        &mut self,
        components: &mut ComponentRegistry,
    ) -> Result<ArchetypeId, DuplicateComponentError>
    where
        B: Bundle,
    {
        let Self { archetypes, graph } = self;

        let component_ids: Vec<_> = B::register_components(components).into_iter().collect();
        let key = {
            let component_ids = component_ids.iter().copied();
            try_collect_components(component_ids, ArchetypeKey::insert, Clone::clone)?
        };

        let f = |components| ArchetypeStorage::of::<B>(components);
        Self::register(archetypes, graph, components, &component_ids, key, f)
    }

    #[inline]
    pub fn register_archetype_from<I>(
        &mut self,
        components: &ComponentRegistry,
        component_ids: I,
    ) -> Result<ArchetypeId, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let Self { archetypes, graph } = self;

        let component_ids: Vec<_> = component_ids.into_iter().collect();
        let key = {
            let component_ids = component_ids.iter().copied();
            try_collect_components(component_ids, ArchetypeKey::insert, Clone::clone)?
        };

        Self::register_from_slice(archetypes, graph, components, &component_ids, key)
    }

    #[inline]
    fn register<C, F, E>(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        components: C,
        component_ids: &[ComponentId],
        key: ArchetypeKey,
        f: F,
    ) -> Result<ArchetypeId, E>
    where
        C: Borrow<ComponentRegistry>,
        F: FnOnce(C) -> Result<ArchetypeStorage, E>,
    {
        assert!(
            !component_ids.is_empty(),
            "archetype should contain at least one component",
        );

        let archetype_id = Self::find_archetype(archetypes, &key);
        let (before, archetype_to) = if let Some(archetype_id) = archetype_id {
            (Vec::new(), archetype_id)
        } else {
            let borrow = components.borrow();
            let before = Self::register_before(archetypes, graph, borrow, component_ids, &key);
            let storage = f(components)?;
            let archetype_id = Self::register_one(archetypes, graph, key, storage);
            (before, archetype_id)
        };

        for (archetype_from, component_id) in before {
            let archetype_from = archetype_from.into_u32().into();
            let archetype_to = archetype_to.into_u32().into();
            let _ = graph.update_edge(archetype_from, archetype_to, component_id);
        }
        Ok(archetype_to)
    }

    #[inline]
    fn register_from_slice(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        components: &ComponentRegistry,
        component_ids: &[ComponentId],
        key: ArchetypeKey,
    ) -> Result<ArchetypeId, ArchetypeError> {
        let f = |components| ArchetypeStorage::new(components, component_ids.iter().copied());
        Self::register(archetypes, graph, components, component_ids, key, f)
    }

    #[inline]
    fn register_before(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        components: &ComponentRegistry,
        component_ids: &[ComponentId],
        key: &ArchetypeKey,
    ) -> Vec<(ArchetypeId, ComponentId)> {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn difference_fail(key: &ArchetypeKey, sub_key: &ArchetypeKey) -> ! {
            unreachable!("difference of {key:?} from {sub_key:?} should have exactly one element")
        }

        let len = component_ids.len();
        if len <= 1 {
            return Vec::new();
        }

        let register_subset = |component_ids: Vec<_>| {
            let sub_key = component_ids.iter().copied().collect();
            let [component_id] = key
                .difference(&sub_key)
                .copied()
                .collect_array()
                .unwrap_or_else(|| difference_fail(key, &sub_key));
            let archetype_id =
                Self::register_from_slice(archetypes, graph, components, &component_ids, sub_key)
                    .expect("components should be unique & registered");
            (archetype_id, component_id)
        };
        component_ids
            .iter()
            .copied()
            .combinations(len - 1)
            .map(register_subset)
            .collect()
    }

    #[inline]
    fn register_one(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        key: ArchetypeKey,
        storage: ArchetypeStorage,
    ) -> ArchetypeId {
        let index = archetypes.len();
        let id = archetype_id_from_usize(index);

        let info = ArchetypeInfo { id, storage };
        if archetypes.insert(key, info).is_some() {
            unreachable!("duplicate archetype registration")
        }

        let node_index = graph.add_node(()).index();
        if index != node_index {
            unreachable!("archetype index {index} must be equal to node index {node_index}")
        }

        id
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
    pub fn get_archetype_info(&self, id: ArchetypeId) -> Option<&ArchetypeInfo> {
        let Self { archetypes, .. } = self;
        get_archetype_info(archetypes, id)
    }

    #[inline]
    pub unsafe fn get_archetype_info_mut(&mut self, id: ArchetypeId) -> Option<&mut ArchetypeInfo> {
        let Self { archetypes, .. } = self;
        get_archetype_info_mut(archetypes, id)
    }

    #[inline]
    pub fn archetype_id_from<I>(
        &self,
        component_ids: I,
    ) -> Result<Option<ArchetypeId>, DuplicateComponentError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let Self { archetypes, .. } = self;

        let key = try_collect_components(component_ids, ArchetypeKey::insert, Clone::clone)?;
        let archetype_id = Self::find_archetype(archetypes, &key);
        Ok(archetype_id)
    }

    #[inline]
    pub fn archetype_id<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<Option<ArchetypeId>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;

        let component_ids = B::get_components(components);
        let key = try_collect_opt_components(component_ids, ArchetypeKey::insert, Clone::clone)?;
        let archetype_id = Self::find_archetype(archetypes, &key);
        Ok(archetype_id)
    }

    #[inline]
    fn find_archetype(archetypes: &Archetypes, key: &ArchetypeKey) -> Option<ArchetypeId> {
        let (index, _, _) = archetypes.get_full(key)?;
        Some(archetype_id_from_usize(index))
    }

    #[inline]
    pub fn archetype_ids(&self) -> ArchetypeIds {
        let len = self.len();
        let len = archetype_id_from_usize(len).into_u32();
        ArchetypeIds { inner: 0..len }
    }

    #[inline]
    pub fn archetypes_before(&self, id: ArchetypeId) -> ArchetypesBefore<'_> {
        let Self { archetypes, graph } = self;
        ArchetypesBefore::new(archetypes, graph, id, true)
    }

    #[inline]
    pub fn archetypes_before_inclusive(&self, id: ArchetypeId) -> ArchetypesBefore<'_> {
        let Self { archetypes, graph } = self;
        ArchetypesBefore::new(archetypes, graph, id, false)
    }

    #[inline]
    pub fn archetypes_after(&self, id: ArchetypeId) -> ArchetypesAfter<'_> {
        let Self { archetypes, graph } = self;
        ArchetypesAfter::new(archetypes, graph, id, true)
    }

    #[inline]
    pub fn archetypes_after_inclusive(&self, id: ArchetypeId) -> ArchetypesAfter<'_> {
        let Self { archetypes, graph } = self;
        ArchetypesAfter::new(archetypes, graph, id, false)
    }

    #[inline]
    pub fn get_bundle<B>(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<BundleRefs<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let location = EntityArchetypeLocation::Unknown;
        self.get_bundle_with::<B>(components, entity, location)
    }

    #[inline]
    #[track_caller]
    pub fn get_bundle_with<B>(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
        location: EntityArchetypeLocation,
    ) -> Result<BundleRefs<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;
        let Some(archetype_id) = Self::find_archetype_with_entity(archetypes, entity, location)
        else {
            let component_ids = B::get_components(components);
            let error = Self::make_incompatible_bundle_error(component_ids);
            return Err(error);
        };

        let info = unwrap_archetype_info(archetypes, archetype_id);
        let Some(refs) = info.storage().get_bundle::<B>(components, entity)? else {
            let component_ids = B::get_components(components);
            let error = Self::make_incompatible_bundle_error(component_ids);
            return Err(error);
        };
        Ok(refs)
    }

    #[inline]
    pub fn get_bundle_mut<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<BundleRefsMut<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let location = EntityArchetypeLocation::Unknown;
        self.get_bundle_mut_with::<B>(components, entity, location)
    }

    #[inline]
    #[track_caller]
    pub fn get_bundle_mut_with<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
        location: EntityArchetypeLocation,
    ) -> Result<BundleRefsMut<'_, B>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;
        let Some(archetype_id) = Self::find_archetype_with_entity(archetypes, entity, location)
        else {
            let component_ids = B::get_components(components);
            let error = Self::make_incompatible_bundle_error(component_ids);
            return Err(error);
        };

        let info = unwrap_archetype_info_mut(archetypes, archetype_id);
        let Some(refs) = info.storage.get_bundle_mut::<B>(components, entity)? else {
            let component_ids = B::get_components(components);
            let error = Self::make_incompatible_bundle_error(component_ids);
            return Err(error);
        };
        Ok(refs)
    }

    #[inline]
    fn make_incompatible_bundle_error<I>(component_ids: I) -> IncompatibleArchetypeError
    where
        I: IntoIterator<Item = Option<ComponentId>>,
    {
        let result = try_collect_opt_components(component_ids, IndexSet::<_>::insert, Clone::clone);
        let component_ids = match result {
            Ok(component_ids) => component_ids,
            Err(error) => return error.into(),
        };

        let Some(component_id) = component_ids.into_iter().next() else {
            unreachable!("bundle should contain at least one component")
        };
        MissingComponentError::new(component_id).into()
    }

    #[inline]
    pub fn bundles<'ctx, B>(
        &self,
        components: &'ctx ComponentRegistry,
    ) -> Result<Bundles<'_, 'ctx, B>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;
        Bundles::new(archetypes, components)
    }

    #[inline]
    pub fn bundles_mut<'ctx, B>(
        &mut self,
        components: &'ctx ComponentRegistry,
    ) -> Result<BundlesMut<'_, 'ctx, B>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;
        BundlesMut::new(archetypes, components)
    }

    #[inline]
    pub fn compatible_archetypes<I>(
        &self,
        component_ids: I,
    ) -> Result<CompatibleArchetypes<'_>, DuplicateComponentError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let Self { archetypes, .. } = self;
        CompatibleArchetypes::new(archetypes, component_ids)
    }

    #[inline]
    pub fn compatible_archetypes_of<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<CompatibleArchetypes<'_>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;
        CompatibleArchetypes::of::<B>(archetypes, components)
    }

    #[inline]
    pub unsafe fn compatible_archetypes_mut<I>(
        &mut self,
        component_ids: I,
    ) -> Result<CompatibleArchetypesMut<'_>, DuplicateComponentError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let Self { archetypes, .. } = self;
        CompatibleArchetypesMut::new(archetypes, component_ids)
    }

    #[inline]
    pub unsafe fn compatible_archetypes_mut_of<B>(
        &mut self,
        components: &ComponentRegistry,
    ) -> Result<CompatibleArchetypesMut<'_>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;
        CompatibleArchetypesMut::of::<B>(archetypes, components)
    }

    #[inline]
    pub fn insert_bundle_exact<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        value: B,
    ) -> Result<(), InsertBundleExactError<B>>
    where
        B: Bundle,
    {
        let location = EntityArchetypeLocation::Unknown;
        self.insert_bundle_exact_with::<B>(components, entity, value, location)?;
        Ok(())
    }

    #[inline]
    #[track_caller]
    pub fn insert_bundle_exact_with<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        value: B,
        location: EntityArchetypeLocation,
    ) -> Result<ArchetypeId, InsertBundleExactError<B>>
    where
        B: Bundle,
    {
        let Self { archetypes, graph } = self;

        let component_ids: Vec<_> = B::register_components(components).into_iter().collect();
        if let Err(error) = try_collect_components(
            component_ids.iter().copied(),
            ArchetypeKey::insert,
            Clone::clone,
        ) {
            let kind = error.into();
            return Err(InsertBundleExactError { value, kind });
        }

        let old_archetype = Self::find_archetype_with_entity_and_without_components(
            archetypes,
            &component_ids,
            entity,
            location,
        );
        let old_archetype = match old_archetype {
            Ok(old_archetype) => old_archetype,
            Err(error) => {
                let kind = error.into();
                return Err(InsertBundleExactError { value, kind });
            }
        };
        let new_archetype = Self::register_archetype_with_components(
            graph,
            archetypes,
            components,
            old_archetype,
            &component_ids,
        );

        let old_fields = Self::move_out_of_archetype_by_entity(archetypes, old_archetype, entity)
            .map(|bundle| {
                bundle
                    .into_iter()
                    .map(|component| component.expect("component should be allocated successfully"))
                    .collect::<IndexSet<_>>()
            });
        let Some(mut old_fields) = old_fields else {
            let info = unwrap_archetype_info_mut(archetypes, new_archetype);
            if let Err(error) = info.storage.insert_bundle(components, entity, value) {
                let error = error.reason;
                unreachable!("failed to insert {entity} into {new_archetype}: {error}")
            }
            return Ok(new_archetype);
        };

        let fields = ErasedBundle::<StorageMeta>::try_from(components, value)
            .map_err(|error| error.reason)
            .expect("bundle compatibility should have been already checked")
            .into_iter()
            .map(|component| component.expect("component should be allocated successfully"));

        // TODO: add new method for erased bundle to replace some of the components
        fields.for_each(|field| {
            let component_id = field.component_id();
            if old_fields.replace(field).is_some() {
                unreachable!("duplicated {component_id}")
            }
        });

        assert!(
            !old_fields.is_empty(),
            "bundle should contain at least one component",
        );
        let bundle = ErasedBundle::from_components(old_fields)
            .expect("erased bundle should be created successfully");
        Self::set_in_archetype_by_entity(archetypes, Some(new_archetype), entity, bundle);

        Ok(new_archetype)
    }

    #[inline]
    pub fn insert_bundle<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        value: B,
    ) -> Result<(), InsertBundleError<B>>
    where
        B: Bundle,
    {
        let location = EntityArchetypeLocation::Unknown;
        self.insert_bundle_with::<B>(components, entity, value, location)?;
        Ok(())
    }

    #[inline]
    #[track_caller]
    pub fn insert_bundle_with<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        value: B,
        location: EntityArchetypeLocation,
    ) -> Result<ArchetypeId, InsertBundleError<B>>
    where
        B: Bundle,
    {
        let Self { archetypes, graph } = self;

        let component_ids: Vec<_> = B::register_components(components).into_iter().collect();
        if let Err(reason) = try_collect_components(
            component_ids.iter().copied(),
            ArchetypeKey::insert,
            Clone::clone,
        ) {
            return Err(InsertBundleError { value, reason });
        }

        let old_archetype = Self::find_archetype_with_entity(archetypes, entity, location);
        let new_archetype = Self::register_archetype_with_components(
            graph,
            archetypes,
            components,
            old_archetype,
            &component_ids,
        );

        let old_fields = Self::move_out_of_archetype_by_entity(archetypes, old_archetype, entity)
            .map(|bundle| {
                bundle
                    .into_iter()
                    .map(|component| component.expect("component should be allocated successfully"))
                    .collect::<IndexSet<_>>()
            });
        let Some(mut old_fields) = old_fields else {
            let info = unwrap_archetype_info_mut(archetypes, new_archetype);
            if let Err(error) = info.storage.insert_bundle(components, entity, value) {
                let error = error.reason;
                unreachable!("failed to insert {entity} into {new_archetype}: {error}")
            }
            return Ok(new_archetype);
        };

        let fields = ErasedBundle::<StorageMeta>::try_from(components, value)
            .map_err(|error| error.reason)
            .expect("bundle compatibility should have been already checked")
            .into_iter()
            .map(|component| component.expect("component should be allocated successfully"));

        // TODO: add new method for erased bundle to replace some of the components
        fields.map(|field| old_fields.replace(field)).for_each(drop);

        assert!(
            !old_fields.is_empty(),
            "bundle should contain at least one component",
        );
        let bundle = ErasedBundle::from_components(old_fields)
            .expect("erased bundle should be created successfully");
        Self::set_in_archetype_by_entity(archetypes, Some(new_archetype), entity, bundle);

        Ok(new_archetype)
    }

    #[inline]
    pub fn remove_bundle_exact<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Result<B, RemoveBundleExactError>
    where
        B: Bundle,
    {
        let location = EntityArchetypeLocation::Unknown;
        let (value, _) = self.remove_bundle_exact_with::<B>(components, entity, location)?;
        Ok(value)
    }

    #[inline]
    #[track_caller]
    pub fn remove_bundle_exact_with<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        location: EntityArchetypeLocation,
    ) -> Result<(B, EntityArchetype), RemoveBundleExactError>
    where
        B: Bundle,
    {
        let Self { archetypes, graph } = self;

        let component_ids: Vec<_> = B::register_components(components).into_iter().collect();
        try_collect_components(
            component_ids.iter().copied(),
            ArchetypeKey::insert,
            Clone::clone,
        )?;

        let old_archetype = Self::find_archetype_with_entity_and_with_components(
            archetypes,
            &component_ids,
            entity,
            location,
        )?;
        let Some(old_archetype) = old_archetype else {
            let &[component_id, ..] = component_ids.as_slice() else {
                unreachable!("bundle should contain at least one component")
            };
            return Err(MissingComponentError::new(component_id).into());
        };
        let new_archetype = Self::register_archetype_without_components(
            graph,
            archetypes,
            components,
            old_archetype,
            &component_ids,
        );

        if new_archetype.is_none() {
            let info = unwrap_archetype_info_mut(archetypes, old_archetype);
            let value = info
                .storage
                .remove_bundle::<B>(components, entity)
                .expect("archetype should be compatible")
                .expect("storage should contain data of given entity");
            return Ok((value, new_archetype));
        }

        let mut old_fields =
            Self::move_out_of_archetype_by_entity(archetypes, Some(old_archetype), entity)
                .expect("old archetype should exist")
                .into_iter()
                .map(|component| component.expect("component should be allocated successfully"))
                .collect::<IndexSet<_>>();

        // TODO: add new method for erased bundle to take out some of the components
        let fields = component_ids.iter().copied().map(|component_id| {
            old_fields
                .swap_take(&component_id)
                .unwrap_or_else(|| unreachable!("{component_id} should exist"))
        });
        let value = B::from_erased(components, fields)
            .expect("input fields should be compatible with the bundle");

        assert!(
            !old_fields.is_empty(),
            "bundle should contain at least one component",
        );
        let bundle = ErasedBundle::from_components(old_fields)
            .expect("erased bundle should be created successfully");
        Self::set_in_archetype_by_entity(archetypes, new_archetype, entity, bundle);

        Ok((value, new_archetype))
    }

    #[inline]
    pub fn remove_bundle<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Result<(), DuplicateComponentError>
    where
        B: Bundle,
    {
        let location = EntityArchetypeLocation::Unknown;
        let _ = self.remove_bundle_with::<B>(components, entity, location)?;
        Ok(())
    }

    #[inline]
    #[track_caller]
    pub fn remove_bundle_with<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        location: EntityArchetypeLocation,
    ) -> Result<EntityArchetype, DuplicateComponentError>
    where
        B: Bundle,
    {
        let Self { archetypes, graph } = self;

        let component_ids: Vec<_> = B::register_components(components).into_iter().collect();
        try_collect_components(
            component_ids.iter().copied(),
            ArchetypeKey::insert,
            Clone::clone,
        )?;

        let old_archetype = Self::find_archetype_with_entity(archetypes, entity, location);
        let Some(old_archetype) = old_archetype else {
            return Ok(None);
        };
        let new_archetype = Self::register_archetype_without_components(
            graph,
            archetypes,
            components,
            old_archetype,
            &component_ids,
        );

        if new_archetype.is_none() {
            let info = unwrap_archetype_info_mut(archetypes, old_archetype);
            if !info.storage.destroy(entity) {
                unreachable!("{entity} should exist in {old_archetype}")
            }
            return Ok(new_archetype);
        }

        let mut old_fields =
            Self::move_out_of_archetype_by_entity(archetypes, Some(old_archetype), entity)
                .expect("old archetype should exist")
                .into_iter()
                .map(|component| component.expect("component should be allocated successfully"))
                .collect::<IndexSet<_>>();

        // TODO: add new method for erased bundle to remove some of the components
        component_ids
            .iter()
            .map(|component_id| old_fields.swap_take(component_id))
            .for_each(drop);

        assert!(
            !old_fields.is_empty(),
            "bundle should contain at least one component",
        );
        let bundle = ErasedBundle::from_components(old_fields)
            .expect("erased bundle should be created successfully");
        Self::set_in_archetype_by_entity(archetypes, new_archetype, entity, bundle);

        Ok(new_archetype)
    }

    #[inline]
    pub fn destroy(&mut self, entity: Entity, location: EntityArchetypeLocation) -> bool {
        let Self { archetypes, .. } = self;

        let Some(archetype_id) = Self::find_archetype_with_entity(archetypes, entity, location)
        else {
            return false;
        };
        let info = unwrap_archetype_info_mut(archetypes, archetype_id);
        if !info.storage.destroy(entity) {
            unreachable!("{entity} should exist in {archetype_id}");
        }
        true
    }

    #[inline]
    fn set_in_archetype_by_entity<T>(
        archetypes: &mut Archetypes,
        archetype_id: Option<ArchetypeId>,
        entity: Entity,
        bundle: ErasedBundleKind<T>,
    ) where
        T: ErasedArchetypeKind<Meta = StorageMeta>,
    {
        let Some(archetype_id) = archetype_id else {
            return;
        };

        let info = unwrap_archetype_info_mut(archetypes, archetype_id);
        if let Err(error) = info.storage.insert(entity, bundle) {
            unreachable!("failed to insert {entity} into {archetype_id}: {error}");
        }
    }

    #[inline]
    fn move_out_of_archetype_by_entity(
        archetypes: &mut Archetypes,
        archetype_id: Option<ArchetypeId>,
        entity: Entity,
    ) -> Option<ErasedBorrowedBundle<'_, StorageMeta>> {
        let archetype_id = archetype_id?;

        let info = unwrap_archetype_info_mut(archetypes, archetype_id);
        let Some(bundle) = info.storage.remove(entity) else {
            unreachable!("{entity} should exist in {archetype_id}")
        };
        Some(bundle)
    }

    #[inline]
    fn find_archetype_with_entity_and_without_components(
        archetypes: &Archetypes,
        component_ids: &[ComponentId],
        entity: Entity,
        location: EntityArchetypeLocation,
    ) -> Result<Option<ArchetypeId>, AlreadyHasComponentError> {
        let Some(archetype_id) = Self::find_archetype_with_entity(archetypes, entity, location)
        else {
            return Ok(None);
        };

        let key = unwrap_archetype_key(archetypes, archetype_id);
        for &component_id in component_ids {
            if key.contains(&component_id) {
                return Err(AlreadyHasComponentError::new(component_id));
            }
        }

        Ok(Some(archetype_id))
    }

    #[inline]
    fn find_archetype_with_entity_and_with_components(
        archetypes: &Archetypes,
        component_ids: &[ComponentId],
        entity: Entity,
        location: EntityArchetypeLocation,
    ) -> Result<Option<ArchetypeId>, MissingComponentError> {
        let Some(archetype_id) = Self::find_archetype_with_entity(archetypes, entity, location)
        else {
            return Ok(None);
        };

        let key = unwrap_archetype_key(archetypes, archetype_id);
        for &component_id in component_ids {
            if !key.contains(&component_id) {
                return Err(MissingComponentError::new(component_id));
            }
        }

        Ok(Some(archetype_id))
    }

    #[inline]
    fn find_archetype_with_entity(
        archetypes: &Archetypes,
        entity: Entity,
        location: EntityArchetypeLocation,
    ) -> Option<ArchetypeId> {
        if let EntityArchetypeLocation::Known(archetype_id) = location {
            let archetype_id = archetype_id?;
            let info = unwrap_archetype_info(archetypes, archetype_id);
            if !info.storage().contains(entity) {
                unreachable!("{archetype_id} should contain {entity}");
            }
            return Some(archetype_id);
        }

        archetypes
            .values()
            .position(|info| info.storage().contains(entity))
            .map(archetype_id_from_usize)
    }

    #[inline]
    fn register_archetype_with_components(
        graph: &mut Graph,
        archetypes: &mut Archetypes,
        components: &ComponentRegistry,
        archetype_id: Option<ArchetypeId>,
        component_ids: &[ComponentId],
    ) -> ArchetypeId {
        let Some(archetype_id) = archetype_id else {
            let key = component_ids.iter().copied().collect();
            return Self::register_from_slice(archetypes, graph, components, component_ids, key)
                .expect("components should be unique & registered");
        };
        if let &[component_id] = component_ids
            && let Some(archetype_id) =
                Self::find_archetype_after(graph, archetype_id, component_id)
        {
            return archetype_id;
        }

        let info = unwrap_archetype_info(archetypes, archetype_id);
        let component_ids: Vec<_> = info
            .storage()
            .component_ids()
            .chain(component_ids.iter().copied())
            .sorted_unstable_by_key(|&component_id| {
                components
                    .get_component_info(component_id)
                    .map(|info| info.descriptor().layout().align())
            })
            .dedup()
            .collect();
        let key = component_ids.iter().copied().collect();
        Self::register_from_slice(archetypes, graph, components, &component_ids, key)
            .expect("components should be unique & registered")
    }

    #[inline]
    fn register_archetype_without_components(
        graph: &mut Graph,
        archetypes: &mut Archetypes,
        components: &ComponentRegistry,
        archetype_id: ArchetypeId,
        component_ids: &[ComponentId],
    ) -> Option<ArchetypeId> {
        if let &[component_id] = component_ids
            && let Some(archetype_id) =
                Self::find_archetype_before(graph, archetype_id, component_id)
        {
            return Some(archetype_id);
        }

        let info = unwrap_archetype_info(archetypes, archetype_id);
        let archetype_component_ids = info.storage().component_ids();
        if archetype_component_ids.len() <= 1 {
            return None;
        }

        let component_ids: ArchetypeKey = component_ids.iter().copied().collect();
        let component_ids: Vec<_> = archetype_component_ids
            .filter(|component_id| !component_ids.contains(component_id))
            .collect();
        if component_ids.is_empty() {
            return None;
        }

        let key = component_ids.iter().copied().collect();
        let archetype_id =
            Self::register_from_slice(archetypes, graph, components, &component_ids, key)
                .expect("components should be unique & registered");
        Some(archetype_id)
    }

    #[inline]
    fn find_archetype_before(
        graph: &Graph,
        archetype_id: ArchetypeId,
        component_id: ComponentId,
    ) -> Option<ArchetypeId> {
        graph
            .edges_directed(archetype_id.into_u32().into(), Direction::Incoming)
            .find(|edge| *edge.weight() == component_id)
            .map(|edge| archetype_id_from_usize(edge.source().index()))
    }

    #[inline]
    fn find_archetype_after(
        graph: &Graph,
        archetype_id: ArchetypeId,
        component_id: ComponentId,
    ) -> Option<ArchetypeId> {
        graph
            .edges_directed(archetype_id.into_u32().into(), Direction::Outgoing)
            .find(|edge| *edge.weight() == component_id)
            .map(|edge| archetype_id_from_usize(edge.target().index()))
    }
}

impl Debug for ArchetypeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, graph } = self;

        graph_dot_scoped(archetypes, graph, |graph| {
            f.debug_struct("ArchetypeRegistry")
                .field("archetypes", archetypes)
                .field("graph", graph)
                .finish()
        })
    }
}

#[inline]
fn graph_dot_scoped<F, O>(archetypes: &Archetypes, graph: &Graph, f: F) -> O
where
    F: FnOnce(&Dot<&Graph>) -> O,
{
    let config = [
        DotConfig::NodeNoLabel,
        DotConfig::EdgeNoLabel,
        DotConfig::RankDir(RankDir::LR),
    ];
    let node_attrs = |_, (index, &()): (NodeIndex<_>, _)| {
        let archetype_id = archetype_id_from_usize(index.index());
        let info = unwrap_archetype_info(archetypes, archetype_id);
        let component_ids = info.storage().component_ids();
        format!(r#"shape=box label="{archetype_id:?}\n{component_ids:?}" "#)
    };
    let edge_attrs = |_, edge: EdgeReference<'_, _, _>| {
        let component_id = edge.weight();
        format!(r#"label="{component_id:?}" "#)
    };
    let dot = Dot::with_attr_getters(graph, &config, &edge_attrs, &node_attrs);
    f(&dot)
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ArchetypeIds {
    inner: Range<u32>,
}

impl ArchetypeIds {
    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { inner } = self;
        inner.is_empty()
    }
}

impl Debug for ArchetypeIds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let Range { start, end } = *inner;
        let ids = archetype_id_trusted(start)..archetype_id_trusted(end);
        f.debug_struct("ArchetypeIds").field("ids", &ids).finish()
    }
}

impl Iterator for ArchetypeIds {
    type Item = ArchetypeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(archetype_id_trusted)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(archetype_id_trusted)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(archetype_id_trusted)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(archetype_id_trusted)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(archetype_id_trusted)
    }

    #[inline]
    fn is_sorted(self) -> bool {
        let Self { inner } = self;
        inner.is_sorted()
    }
}

impl DoubleEndedIterator for ArchetypeIds {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(archetype_id_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(archetype_id_trusted)
    }
}

impl ExactSizeIterator for ArchetypeIds {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for ArchetypeIds {}

#[derive(Clone)]
pub struct ArchetypesBefore<'a> {
    archetypes: &'a Archetypes,
    walker: WalkerIter<Bfs<NodeIndex<u32>, <Graph as Visitable>::Map>, Reversed<&'a Graph>>,
    archetype_id: ArchetypeId,
    exclusive: bool,
}

impl<'a> ArchetypesBefore<'a> {
    #[inline]
    fn new(
        archetypes: &'a Archetypes,
        graph: &'a Graph,
        archetype_id: ArchetypeId,
        exclusive: bool,
    ) -> Self {
        let start = archetype_id.into_u32().into();
        let graph = Reversed(graph);
        let walker = Bfs::new(graph, start).iter(graph);
        Self {
            archetypes,
            walker,
            archetype_id,
            exclusive,
        }
    }

    #[inline]
    pub fn is_inclusive(&self) -> bool {
        let Self { exclusive, .. } = self;
        !exclusive
    }
}

impl Debug for ArchetypesBefore<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            archetypes,
            walker,
            archetype_id,
            exclusive,
        } = self;

        let graph = walker.context().0;
        let inclusive = &!exclusive;
        graph_dot_scoped(archetypes, graph, |graph| {
            f.debug_struct("ArchetypesBefore")
                .field("archetypes", archetypes)
                .field("graph", graph)
                .field("archetype_id", archetype_id)
                .field("inclusive", inclusive)
                .finish()
        })
    }
}

impl<'a> Iterator for ArchetypesBefore<'a> {
    type Item = &'a ArchetypeInfo;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut walker,
            archetypes,
            archetype_id,
            exclusive,
        } = *self;

        let index = if exclusive {
            walker.find(|index| index.index() != archetype_id_into_usize(archetype_id))
        } else {
            walker.next()
        }?;

        let archetype_id = archetype_id_from_usize(index.index());
        let info = unwrap_archetype_info(archetypes, archetype_id);
        Some(info)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self {
            archetypes,
            exclusive,
            ..
        } = *self;

        let skip_count = usize::from(exclusive);
        let upper = archetypes.len().saturating_sub(skip_count);
        (0, Some(upper))
    }
}

#[derive(Clone)]
pub struct ArchetypesAfter<'a> {
    archetypes: &'a Archetypes,
    walker: WalkerIter<Bfs<NodeIndex<u32>, <Graph as Visitable>::Map>, &'a Graph>,
    archetype_id: ArchetypeId,
    exclusive: bool,
}

impl<'a> ArchetypesAfter<'a> {
    #[inline]
    fn new(
        archetypes: &'a Archetypes,
        graph: &'a Graph,
        archetype_id: ArchetypeId,
        exclusive: bool,
    ) -> Self {
        let start = archetype_id.into_u32().into();
        let walker = Bfs::new(graph, start).iter(graph);
        Self {
            archetypes,
            walker,
            archetype_id,
            exclusive,
        }
    }

    #[inline]
    pub fn is_inclusive(&self) -> bool {
        let Self { exclusive, .. } = self;
        !exclusive
    }
}

impl Debug for ArchetypesAfter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            archetypes,
            walker,
            archetype_id,
            exclusive,
        } = self;

        let graph = walker.context();
        let inclusive = &!exclusive;
        graph_dot_scoped(archetypes, graph, |graph| {
            f.debug_struct("ArchetypesBefore")
                .field("archetypes", archetypes)
                .field("graph", graph)
                .field("archetype_id", archetype_id)
                .field("inclusive", inclusive)
                .finish()
        })
    }
}

impl<'a> Iterator for ArchetypesAfter<'a> {
    type Item = &'a ArchetypeInfo;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut walker,
            archetypes,
            archetype_id,
            exclusive,
        } = *self;

        let index = if exclusive {
            walker.find(|index| index.index() != archetype_id_into_usize(archetype_id))
        } else {
            walker.next()
        }?;

        let archetype_id = archetype_id_from_usize(index.index());
        let info = unwrap_archetype_info(archetypes, archetype_id);
        Some(info)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self {
            archetypes,
            exclusive,
            ..
        } = *self;

        let skip_count = usize::from(exclusive);
        let upper = archetypes.len().saturating_sub(skip_count);
        (0, Some(upper))
    }
}

#[derive(Clone)]
pub struct CompatibleArchetypes<'a> {
    component_ids: Box<[ComponentId]>,
    infos: IndexMapValues<'a, ArchetypeKey, ArchetypeInfo>,
}

impl<'a> CompatibleArchetypes<'a> {
    #[inline]
    fn new<I>(archetypes: &'a Archetypes, component_ids: I) -> Result<Self, DuplicateComponentError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let component_ids =
            try_collect_components(component_ids, IndexSet::<_>::insert, Clone::clone)?
                .into_iter()
                .collect();
        let infos = archetypes.values();
        Ok(Self {
            component_ids,
            infos,
        })
    }

    #[inline]
    fn of<B>(
        archetypes: &'a Archetypes,
        components: &ComponentRegistry,
    ) -> Result<Self, ArchetypeError>
    where
        B: Bundle,
    {
        let component_ids = B::get_components(components);
        let component_ids =
            try_collect_opt_components(component_ids, IndexSet::<_>::insert, Clone::clone)?
                .into_iter()
                .collect();
        let infos = archetypes.values();
        Ok(Self {
            component_ids,
            infos,
        })
    }

    #[inline]
    pub fn component_ids(&self) -> &[ComponentId] {
        let Self { component_ids, .. } = self;
        component_ids
    }
}

impl Debug for CompatibleArchetypes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { component_ids, .. } = self;
        f.debug_struct("CompatibleArchetypes")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}

impl<'a> Iterator for CompatibleArchetypes<'a> {
    type Item = &'a ArchetypeInfo;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref component_ids,
            ref mut infos,
        } = *self;

        infos.find(|info| compatible_archetypes_predicate(info, component_ids))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { infos, .. } = self;
        let (_, upper) = infos.size_hint();
        (0, upper) // can't know a lower bound, due to the predicate
    }

    #[inline]
    fn count(self) -> usize {
        let Self {
            infos,
            ref component_ids,
        } = self;

        infos
            .filter(|info| compatible_archetypes_predicate(info, component_ids))
            .count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let Self {
            infos,
            ref component_ids,
        } = self;

        infos
            .filter(|info| compatible_archetypes_predicate(info, component_ids))
            .fold(init, f)
    }
}

impl DoubleEndedIterator for CompatibleArchetypes<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self {
            ref component_ids,
            ref mut infos,
        } = *self;

        infos.rfind(|info| compatible_archetypes_predicate(info, component_ids))
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let Self {
            ref component_ids,
            infos,
        } = self;

        infos
            .filter(|info| compatible_archetypes_predicate(info, component_ids))
            .rfold(init, f)
    }
}

impl FusedIterator for CompatibleArchetypes<'_> {}

pub struct CompatibleArchetypesMut<'a> {
    component_ids: Box<[ComponentId]>,
    infos: IndexMapValuesMut<'a, ArchetypeKey, ArchetypeInfo>,
}

impl<'a> CompatibleArchetypesMut<'a> {
    #[inline]
    fn new<I>(
        archetypes: &'a mut Archetypes,
        component_ids: I,
    ) -> Result<Self, DuplicateComponentError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let component_ids =
            try_collect_components(component_ids, IndexSet::<_>::insert, Clone::clone)?
                .into_iter()
                .collect();
        let infos = archetypes.values_mut();
        Ok(Self {
            component_ids,
            infos,
        })
    }

    #[inline]
    fn of<B>(
        archetypes: &'a mut Archetypes,
        components: &ComponentRegistry,
    ) -> Result<Self, ArchetypeError>
    where
        B: Bundle,
    {
        let component_ids = B::get_components(components);
        let component_ids =
            try_collect_opt_components(component_ids, IndexSet::<_>::insert, Clone::clone)?
                .into_iter()
                .collect();
        let infos = archetypes.values_mut();
        Ok(Self {
            component_ids,
            infos,
        })
    }

    #[inline]
    pub fn component_ids(&self) -> &[ComponentId] {
        let Self { component_ids, .. } = self;
        component_ids
    }
}

impl Debug for CompatibleArchetypesMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { component_ids, .. } = self;
        f.debug_struct("CompatibleArchetypesMut")
            .field("component_ids", component_ids)
            .finish_non_exhaustive()
    }
}

impl<'a> Iterator for CompatibleArchetypesMut<'a> {
    type Item = &'a mut ArchetypeInfo;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref component_ids,
            ref mut infos,
        } = *self;

        infos.find(|info| compatible_archetypes_predicate(info, component_ids))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { infos, .. } = self;
        let (_, upper) = infos.size_hint();
        (0, upper) // can't know a lower bound, due to the predicate
    }

    #[inline]
    fn count(self) -> usize {
        let Self {
            infos,
            ref component_ids,
        } = self;

        infos
            .filter(|info| compatible_archetypes_predicate(info, component_ids))
            .count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let Self {
            infos,
            ref component_ids,
        } = self;

        infos
            .filter(|info| compatible_archetypes_predicate(info, component_ids))
            .fold(init, f)
    }
}

impl DoubleEndedIterator for CompatibleArchetypesMut<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self {
            ref component_ids,
            ref mut infos,
        } = *self;

        infos.rfind(|info| compatible_archetypes_predicate(info, component_ids))
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let Self {
            ref component_ids,
            infos,
        } = self;

        infos
            .filter(|info| compatible_archetypes_predicate(info, component_ids))
            .rfold(init, f)
    }
}

impl FusedIterator for CompatibleArchetypesMut<'_> {}

#[inline]
fn compatible_archetypes_predicate(info: &ArchetypeInfo, component_ids: &[ComponentId]) -> bool {
    let component_ids = component_ids.iter().copied();
    info.storage()
        .check_compatibility_for(component_ids)
        .is_ok()
}

pub struct Bundles<'a, 'ctx, B>
where
    B: Bundle,
{
    archetypes: CompatibleArchetypes<'a>,
    components: &'ctx ComponentRegistry,
    phantom: PhantomData<fn() -> B>,
}

impl<'a, 'ctx, B> Bundles<'a, 'ctx, B>
where
    B: Bundle,
{
    #[inline]
    fn new(
        archetypes: &'a Archetypes,
        components: &'ctx ComponentRegistry,
    ) -> Result<Self, ArchetypeError> {
        let archetypes = CompatibleArchetypes::of::<B>(archetypes, components)?;
        Ok(Self {
            archetypes,
            components,
            phantom: PhantomData,
        })
    }

    #[inline]
    pub fn archetypes(&self) -> &CompatibleArchetypes<'a> {
        let Self { archetypes, .. } = self;
        archetypes
    }

    #[inline]
    pub fn into_archetypes(self) -> CompatibleArchetypes<'a> {
        let Self { archetypes, .. } = self;
        archetypes
    }
}

impl<B> Debug for Bundles<'_, '_, B>
where
    B: Bundle,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, .. } = self;
        f.debug_struct("Bundles")
            .field("archetypes", archetypes)
            .finish_non_exhaustive()
    }
}

impl<B> Clone for Bundles<'_, '_, B>
where
    B: Bundle,
{
    fn clone(&self) -> Self {
        let Self {
            ref archetypes,
            components,
            phantom,
        } = *self;

        Self {
            archetypes: archetypes.clone(),
            components,
            phantom,
        }
    }
}

impl<'a, 'ctx, B> IntoIterator for Bundles<'a, 'ctx, B>
where
    B: Bundle,
{
    type Item = (Entity, BundleRefs<'a, B>);

    // this actually should be just `FlatMap`,
    // but it cannot be returned because `impl Trait` is unstable in associated types
    type IntoIter = BundlesIntoIter<'a, 'ctx, B>;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            archetypes,
            components,
            ..
        } = self;
        BundlesIntoIter {
            archetypes,
            components,
            inner_front: None,
            inner_back: None,
        }
    }
}

type BundlesIntoIterInner<'a, B> =
    iter::Zip<iter::Copied<slice::Iter<'a, Entity>>, SoaIter<'static, 'a, B>>;

pub struct BundlesIntoIter<'a, 'ctx, B>
where
    B: Bundle,
{
    archetypes: CompatibleArchetypes<'a>,
    components: &'ctx ComponentRegistry,
    inner_front: Option<BundlesIntoIterInner<'a, B>>,
    inner_back: Option<BundlesIntoIterInner<'a, B>>,
}

impl<'a, 'ctx, B> BundlesIntoIter<'a, 'ctx, B>
where
    B: Bundle,
{
    #[inline]
    fn new_inner(
        info: &'a ArchetypeInfo,
        components: &'ctx ComponentRegistry,
    ) -> BundlesIntoIterInner<'a, B> {
        let archetype_id = info.id();
        let Ok((entities, components)) = info.storage().bundles::<B>(components) else {
            unreachable!("{archetype_id} should be compatible with requested bundle")
        };

        let entities = entities.iter().copied();
        let components = SoaSlices::new(B::CONTEXT, components);
        entities.zip(components)
    }
}

impl<'a, B> Debug for BundlesIntoIter<'a, '_, B>
where
    B: Bundle,
    BundlesIntoIterInner<'a, B>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            archetypes,
            inner_front,
            inner_back,
            ..
        } = self;

        f.debug_struct("BundlesIntoIter")
            .field("archetypes", archetypes)
            .field("inner_front", inner_front)
            .field("inner_back", inner_back)
            .finish_non_exhaustive()
    }
}

impl<B> Clone for BundlesIntoIter<'_, '_, B>
where
    B: Bundle,
{
    fn clone(&self) -> Self {
        let Self {
            archetypes,
            components,
            inner_front,
            inner_back,
        } = self;
        Self {
            archetypes: archetypes.clone(),
            components,
            inner_front: inner_front.clone(),
            inner_back: inner_back.clone(),
        }
    }
}

impl<'a, B> Iterator for BundlesIntoIter<'a, '_, B>
where
    B: Bundle,
{
    type Item = (Entity, BundleRefs<'a, B>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            archetypes,
            components,
            inner_front,
            inner_back,
        } = self;

        loop {
            if let item @ Some(_) = and_then_or_clear(inner_front, Iterator::next) {
                return item;
            }
            match archetypes.next() {
                None => return and_then_or_clear(inner_back, Iterator::next),
                Some(info) => *inner_front = Self::new_inner(info, components).into(),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self {
            archetypes,
            inner_front,
            inner_back,
            ..
        } = self;

        let (flo, fhi) = inner_front
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let (blo, bhi) = inner_back
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let lo = flo.saturating_add(blo);

        match (archetypes.size_hint(), fhi, bhi) {
            ((0, Some(0)), Some(a), Some(b)) => (lo, a.checked_add(b)),
            _ => (lo, None),
        }
    }
}

impl<B> DoubleEndedIterator for BundlesIntoIter<'_, '_, B>
where
    B: Bundle,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self {
            archetypes,
            components,
            inner_front,
            inner_back,
        } = self;

        loop {
            if let item @ Some(_) = and_then_or_clear(inner_back, DoubleEndedIterator::next_back) {
                return item;
            }
            match archetypes.next_back() {
                None => return and_then_or_clear(inner_front, DoubleEndedIterator::next_back),
                Some(info) => *inner_back = Self::new_inner(info, components).into(),
            }
        }
    }
}

impl<B> FusedIterator for BundlesIntoIter<'_, '_, B> where B: Bundle {}

pub struct BundlesMut<'a, 'ctx, B>
where
    B: Bundle,
{
    archetypes: CompatibleArchetypesMut<'a>,
    components: &'ctx ComponentRegistry,
    phantom: PhantomData<fn() -> B>,
}

impl<'a, 'ctx, B> BundlesMut<'a, 'ctx, B>
where
    B: Bundle,
{
    #[inline]
    fn new(
        archetypes: &'a mut Archetypes,
        components: &'ctx ComponentRegistry,
    ) -> Result<Self, ArchetypeError> {
        let archetypes = CompatibleArchetypesMut::of::<B>(archetypes, components)?;
        Ok(Self {
            archetypes,
            components,
            phantom: PhantomData,
        })
    }

    #[inline]
    pub fn archetypes(&self) -> &CompatibleArchetypesMut<'a> {
        let Self { archetypes, .. } = self;
        archetypes
    }

    #[inline]
    pub unsafe fn archetypes_mut(&mut self) -> &mut CompatibleArchetypesMut<'a> {
        let Self { archetypes, .. } = self;
        archetypes
    }

    #[inline]
    pub unsafe fn into_archetypes(self) -> CompatibleArchetypesMut<'a> {
        let Self { archetypes, .. } = self;
        archetypes
    }
}

impl<B> Debug for BundlesMut<'_, '_, B>
where
    B: Bundle,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, .. } = self;
        f.debug_struct("BundlesMut")
            .field("archetypes", archetypes)
            .finish_non_exhaustive()
    }
}

impl<'a, 'ctx, B> IntoIterator for BundlesMut<'a, 'ctx, B>
where
    B: Bundle,
{
    type Item = (Entity, BundleRefsMut<'a, B>);

    // this actually should be just `FlatMap`,
    // but it cannot be returned because `impl Trait` is unstable in associated types
    type IntoIter = BundlesMutIntoIter<'a, 'ctx, B>;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            archetypes,
            components,
            ..
        } = self;
        BundlesMutIntoIter {
            archetypes,
            components,
            inner_front: None,
            inner_back: None,
        }
    }
}

type BundlesMutIntoIterInner<'a, B> =
    iter::Zip<iter::Copied<slice::Iter<'a, Entity>>, SoaIterMut<'static, 'a, B>>;

pub struct BundlesMutIntoIter<'a, 'ctx, B>
where
    B: Bundle,
{
    archetypes: CompatibleArchetypesMut<'a>,
    components: &'ctx ComponentRegistry,
    inner_front: Option<BundlesMutIntoIterInner<'a, B>>,
    inner_back: Option<BundlesMutIntoIterInner<'a, B>>,
}

impl<'a, 'ctx, B> BundlesMutIntoIter<'a, 'ctx, B>
where
    B: Bundle,
{
    #[inline]
    fn new_inner(
        info: &'a mut ArchetypeInfo,
        components: &'ctx ComponentRegistry,
    ) -> BundlesMutIntoIterInner<'a, B> {
        let archetype_id = info.id();
        let Ok((entities, components)) = info.storage.bundles_mut::<B>(components) else {
            unreachable!("{archetype_id} should be compatible with requested bundle")
        };

        let entities = entities.iter().copied();
        let components = SoaSlicesMut::new(B::CONTEXT, components);
        entities.zip(components)
    }
}

impl<'a, B> Debug for BundlesMutIntoIter<'a, '_, B>
where
    B: Bundle,
    BundlesMutIntoIterInner<'a, B>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            archetypes,
            inner_front,
            inner_back,
            ..
        } = self;

        f.debug_struct("BundlesIntoIter")
            .field("archetypes", archetypes)
            .field("inner_front", inner_front)
            .field("inner_back", inner_back)
            .finish_non_exhaustive()
    }
}

impl<'a, B> Iterator for BundlesMutIntoIter<'a, '_, B>
where
    B: Bundle,
{
    type Item = (Entity, BundleRefsMut<'a, B>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            archetypes,
            components,
            inner_front,
            inner_back,
        } = self;

        loop {
            if let item @ Some(_) = and_then_or_clear(inner_front, Iterator::next) {
                return item;
            }
            match archetypes.next() {
                None => return and_then_or_clear(inner_back, Iterator::next),
                Some(info) => *inner_front = Self::new_inner(info, components).into(),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self {
            archetypes,
            inner_front,
            inner_back,
            ..
        } = self;

        let (flo, fhi) = inner_front
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let (blo, bhi) = inner_back
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let lo = flo.saturating_add(blo);

        match (archetypes.size_hint(), fhi, bhi) {
            ((0, Some(0)), Some(a), Some(b)) => (lo, a.checked_add(b)),
            _ => (lo, None),
        }
    }
}

impl<B> DoubleEndedIterator for BundlesMutIntoIter<'_, '_, B>
where
    B: Bundle,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self {
            archetypes,
            components,
            inner_front,
            inner_back,
        } = self;

        loop {
            if let item @ Some(_) = and_then_or_clear(inner_back, DoubleEndedIterator::next_back) {
                return item;
            }
            match archetypes.next_back() {
                None => return and_then_or_clear(inner_front, DoubleEndedIterator::next_back),
                Some(info) => *inner_back = Self::new_inner(info, components).into(),
            }
        }
    }
}

impl<B> FusedIterator for BundlesMutIntoIter<'_, '_, B> where B: Bundle {}

#[inline]
fn and_then_or_clear<T, U>(opt: &mut Option<T>, f: impl FnOnce(&mut T) -> Option<U>) -> Option<U> {
    let x = f(opt.as_mut()?);
    if x.is_none() {
        *opt = None;
    }
    x
}

#[inline]
fn archetype_id_from_usize(index: usize) -> ArchetypeId {
    let id = index.try_into().expect("`ArchetypeId` overflow");
    archetype_id_trusted(id)
}

#[inline]
fn archetype_id_into_usize(id: ArchetypeId) -> usize {
    let id = id.into_u32();
    id.try_into().expect("`ArchetypeId` overflow")
}

#[inline]
fn archetype_id_trusted(id: u32) -> ArchetypeId {
    unsafe { ArchetypeId::from_u32(id) }
}

#[inline]
fn get_archetype_key(archetypes: &Archetypes, id: ArchetypeId) -> Option<&ArchetypeKey> {
    let index = archetype_id_into_usize(id);
    archetypes.get_index(index).map(|(key, _)| key)
}

#[inline]
#[track_caller]
fn unwrap_archetype_key(archetypes: &Archetypes, id: ArchetypeId) -> &ArchetypeKey {
    let Some(key) = get_archetype_key(archetypes, id) else {
        unreachable!("{id} should exist")
    };
    key
}

#[inline]
fn get_archetype_info(archetypes: &Archetypes, id: ArchetypeId) -> Option<&ArchetypeInfo> {
    let index = archetype_id_into_usize(id);
    archetypes.get_index(index).map(|(_, info)| info)
}

#[inline]
#[track_caller]
fn unwrap_archetype_info(archetypes: &Archetypes, id: ArchetypeId) -> &ArchetypeInfo {
    let Some(info) = get_archetype_info(archetypes, id) else {
        unreachable!("{id} should exist")
    };
    info
}

#[inline]
fn get_archetype_info_mut(
    archetypes: &mut Archetypes,
    id: ArchetypeId,
) -> Option<&mut ArchetypeInfo> {
    let index = archetype_id_into_usize(id);
    archetypes.get_index_mut(index).map(|(_, info)| info)
}

#[inline]
#[track_caller]
fn unwrap_archetype_info_mut(archetypes: &mut Archetypes, id: ArchetypeId) -> &mut ArchetypeInfo {
    let Some(info) = get_archetype_info_mut(archetypes, id) else {
        unreachable!("{id} should exist")
    };
    info
}
