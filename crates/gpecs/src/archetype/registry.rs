use std::{
    borrow::Borrow,
    collections::BTreeSet,
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::Range,
};

use gpecs_soa_erased::field::ErasedField;
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use petgraph::{
    dot::{Config as DotConfig, Dot, RankDir},
    graph::{DiGraph, EdgeReference, NodeIndex},
    visit::EdgeRef,
    Direction,
};

use crate::{
    archetype::{erased::drop_erased_in_place, storage::ArchetypeStorage},
    bundle::Bundle,
    component::registry::{ComponentId, ComponentRegistry},
    entity::Entity,
    soa::traits::DefaultContext,
};

use super::{
    erased::{from_erased_fields, into_erased_fields, ErasedComponents},
    error::{
        AlreadyHasComponentError, DuplicateComponentError, GetComponentsError,
        IncompatibleBundleError, InsertBundleError, InsertBundleExactError, MissingComponentError,
        RemoveBundleExactError,
    },
    utils::{try_collect_component_ids, try_collect_maybe_component_ids},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct ArchetypeId(usize);

impl ArchetypeId {
    #[inline]
    pub const fn index(&self) -> usize {
        let Self(id) = *self;
        id
    }
}

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
        EntityArchetypeLocation::Known(value)
    }
}

impl From<Option<EntityArchetype>> for EntityArchetypeLocation {
    #[inline]
    fn from(value: Option<EntityArchetype>) -> Self {
        EntityArchetypeLocation::from_option(value)
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
    pub fn storage_mut(&mut self) -> &mut ArchetypeStorage {
        let Self { storage, .. } = self;
        storage
    }
}

type ArchetypeKey = BTreeSet<ComponentId>;
type Archetypes = IndexMap<ArchetypeKey, ArchetypeInfo>;
type Graph = DiGraph<(), ComponentId, usize>;

#[derive(Default)]
pub struct ArchetypeRegistry {
    archetypes: Archetypes,
    graph: Graph,
}

impl ArchetypeRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            archetypes: Archetypes::new(),
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
            try_collect_component_ids(component_ids, ArchetypeKey::insert)?
        };

        let f = |components| ArchetypeStorage::of::<B>(components);
        let archetype_id = Self::register(archetypes, graph, components, &component_ids, key, f);
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_archetype_from<I>(
        &mut self,
        components: &ComponentRegistry,
        component_ids: I,
    ) -> Result<ArchetypeId, DuplicateComponentError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let Self { archetypes, graph } = self;

        let component_ids: Vec<_> = component_ids.into_iter().collect();
        let key = {
            let component_ids = component_ids.iter().copied();
            try_collect_component_ids(component_ids, ArchetypeKey::insert)?
        };
        let archetype_id =
            Self::register_from_slice(archetypes, graph, components, &component_ids, key);
        Ok(archetype_id)
    }

    #[inline]
    fn register<C, F>(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        components: C,
        component_ids: &[ComponentId],
        key: ArchetypeKey,
        f: F,
    ) -> ArchetypeId
    where
        C: Borrow<ComponentRegistry>,
        F: FnOnce(C) -> Result<ArchetypeStorage, DuplicateComponentError>,
    {
        let (before, archetype_to) = match Self::find_archetype(archetypes, &key) {
            Some(archetype_id) => (Default::default(), archetype_id),
            None => {
                let borrow = components.borrow();
                let before = Self::register_before(archetypes, graph, borrow, component_ids, &key);
                let Ok(storage) = f(components) else {
                    unreachable!("components should be unique, but got {component_ids:?}")
                };
                let archetype_id = Self::register_one(archetypes, graph, key, storage);
                (before, archetype_id)
            }
        };

        for (archetype_from, component_id) in before {
            let archetype_from = archetype_from.index().into();
            let archetype_to = archetype_to.index().into();
            let _ = graph.update_edge(archetype_from, archetype_to, component_id);
        }
        archetype_to
    }

    #[inline]
    fn register_from_slice(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        components: &ComponentRegistry,
        component_ids: &[ComponentId],
        key: ArchetypeKey,
    ) -> ArchetypeId {
        let f = |components| ArchetypeStorage::new(components, component_ids.iter().copied());
        Self::register(archetypes, graph, components, &component_ids, key, f)
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
            return Default::default();
        }

        let register_subset = |component_ids: Vec<_>| {
            let sub_key = component_ids.iter().copied().collect();
            let [component_id] = key
                .difference(&sub_key)
                .copied()
                .collect_array()
                .unwrap_or_else(|| difference_fail(key, &sub_key));
            let archetype_id =
                Self::register_from_slice(archetypes, graph, components, &component_ids, sub_key);
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
        let id = ArchetypeId(archetypes.len());

        let info = ArchetypeInfo { id, storage };
        if let Some(_) = archetypes.insert(key, info) {
            unreachable!("duplicate archetype registration")
        }

        let index = id.index();
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
        Self::get_info(archetypes, id)
    }

    #[inline]
    fn get_info(archetypes: &Archetypes, id: ArchetypeId) -> Option<&ArchetypeInfo> {
        let index = id.index();
        archetypes.get_index(index).map(|(_, info)| info)
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn get_archetype_info_mut(&mut self, id: ArchetypeId) -> Option<&mut ArchetypeInfo> {
        let Self { archetypes, .. } = self;
        Self::get_info_mut(archetypes, id)
    }

    #[inline]
    fn get_info_mut(archetypes: &mut Archetypes, id: ArchetypeId) -> Option<&mut ArchetypeInfo> {
        let index = id.index();
        archetypes.get_index_mut(index).map(|(_, info)| info)
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

        let key = try_collect_component_ids(component_ids, ArchetypeKey::insert)?;
        let archetype_id = Self::find_archetype(archetypes, &key);
        Ok(archetype_id)
    }

    #[inline]
    pub fn archetype_id<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<Option<ArchetypeId>, GetComponentsError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;

        let component_ids = B::get_components(components);
        let key = try_collect_maybe_component_ids(component_ids, ArchetypeKey::insert)?;
        let archetype_id = Self::find_archetype(archetypes, &key);
        Ok(archetype_id)
    }

    #[inline]
    fn find_archetype(archetypes: &Archetypes, key: &ArchetypeKey) -> Option<ArchetypeId> {
        let (index, _, _) = archetypes.get_full(key)?;
        Some(ArchetypeId(index))
    }

    #[inline]
    pub fn archetype_ids(&self) -> ArchetypeIds {
        let len = self.len();
        ArchetypeIds { inner: 0..len }
    }

    #[inline]
    pub fn get_bundle<B>(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<B::Refs<'_>, IncompatibleBundleError>
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
    ) -> Result<B::Refs<'_>, IncompatibleBundleError>
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

        let Some(info) = Self::get_info(archetypes, archetype_id) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
        let Some(refs) = info.storage().get::<B>(components, entity)? else {
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
    ) -> Result<B::RefsMut<'_>, IncompatibleBundleError>
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
    ) -> Result<B::RefsMut<'_>, IncompatibleBundleError>
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

        let Some(info) = Self::get_info_mut(archetypes, archetype_id) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
        let Some(refs) = info.storage_mut().get_mut::<B>(components, entity)? else {
            let component_ids = B::get_components(components);
            let error = Self::make_incompatible_bundle_error(component_ids);
            return Err(error);
        };
        Ok(refs)
    }

    #[inline]
    fn make_incompatible_bundle_error<I>(component_ids: I) -> IncompatibleBundleError
    where
        I: IntoIterator<Item = Option<ComponentId>>,
    {
        let result = try_collect_maybe_component_ids(component_ids, IndexSet::<_>::insert);
        let component_ids = match result {
            Ok(component_ids) => component_ids,
            Err(error) => return error.into(),
        };

        let Some(component_id) = component_ids.into_iter().next() else {
            unreachable!("bundle should contain at least one component")
        };
        return MissingComponentError::new(component_id).into();
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
        if let Err(error) =
            try_collect_component_ids(component_ids.iter().copied(), ArchetypeKey::insert)
        {
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

        let mut old_fields =
            Self::move_out_of_archetype_by_entity(components, archetypes, old_archetype, entity);
        let fields = {
            let context = DefaultContext::default();
            into_erased_fields::<B>(components, &context, component_ids, value)
        };
        fields.into_iter().for_each(|(component_id, field)| {
            if let Some(_) = old_fields.insert(component_id, field) {
                unreachable!("duplicated component {component_id:?}")
            }
        });

        let new_fields = old_fields;
        let archetype_id = Some(new_archetype);
        Self::set_in_archetype_by_entity(components, archetypes, archetype_id, entity, new_fields);

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
        if let Err(reason) =
            try_collect_component_ids(component_ids.iter().copied(), ArchetypeKey::insert)
        {
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

        let mut old_fields =
            Self::move_out_of_archetype_by_entity(components, archetypes, old_archetype, entity);
        let fields = {
            let context = DefaultContext::default();
            into_erased_fields::<B>(components, &context, component_ids, value)
        };

        let mut fields_to_drop = ErasedComponents::new();
        fields.into_iter().for_each(|(component_id, field)| {
            let Some(to_drop) = old_fields.insert(component_id, field) else {
                return;
            };
            if let Some(_) = fields_to_drop.insert(component_id, to_drop) {
                unreachable!("duplicated component {component_id:?}")
            }
        });
        #[allow(unsafe_code)]
        unsafe {
            Self::drop_erased_in_place(components, fields_to_drop)
        }

        let new_fields = old_fields;
        let archetype_id = Some(new_archetype);
        Self::set_in_archetype_by_entity(components, archetypes, archetype_id, entity, new_fields);

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
        try_collect_component_ids(component_ids.iter().copied(), ArchetypeKey::insert)?;

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

        let old_archetype = Some(old_archetype);
        let mut old_fields =
            Self::move_out_of_archetype_by_entity(components, archetypes, old_archetype, entity);

        let fields = component_ids
            .iter()
            .copied()
            .map(|component_id| {
                let Some(field) = old_fields.swap_remove(&component_id) else {
                    unreachable!("component {component_id:?} should exist")
                };
                (component_id, field)
            })
            .collect();
        #[allow(unsafe_code)]
        let value = unsafe {
            let context = DefaultContext::default();
            from_erased_fields(components, &context, component_ids, fields)
        };

        let new_fields = old_fields;
        Self::set_in_archetype_by_entity(components, archetypes, new_archetype, entity, new_fields);

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
        try_collect_component_ids(component_ids.iter().copied(), ArchetypeKey::insert)?;

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
        if Some(old_archetype) == new_archetype {
            return Ok(new_archetype);
        }

        let old_archetype = Some(old_archetype);
        let mut old_fields =
            Self::move_out_of_archetype_by_entity(components, archetypes, old_archetype, entity);

        let fields_to_drop = component_ids.iter().copied().filter_map(|component_id| {
            let field = old_fields.swap_remove(&component_id)?;
            Some((component_id, field))
        });
        #[allow(unsafe_code)]
        unsafe {
            Self::drop_erased_in_place(components, fields_to_drop)
        }

        let new_fields = old_fields;
        Self::set_in_archetype_by_entity(components, archetypes, new_archetype, entity, new_fields);

        Ok(new_archetype)
    }

    #[inline]
    pub fn destroy_in_place(&mut self, entity: Entity, location: EntityArchetypeLocation) -> bool {
        let Self { archetypes, .. } = self;

        let Some(archetype_id) = Self::find_archetype_with_entity(archetypes, entity, location)
        else {
            return false;
        };
        let Some(info) = Self::get_info_mut(archetypes, archetype_id) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
        if !info.storage_mut().destroy_in_place(entity) {
            unreachable!("entity {entity:?} should exist in archetype {archetype_id:?}");
        }
        true
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn drop_erased_in_place<I, F>(components: &ComponentRegistry, fields: I)
    where
        I: IntoIterator<Item = (ComponentId, F)>,
        F: AsMut<[u8]>,
    {
        let fields = fields.into_iter().map(|(component_id, field)| {
            let Some(info) = components.get_component_info(component_id) else {
                unreachable!("component {component_id:?} should exist")
            };
            (field, info.drop_fn())
        });
        unsafe { drop_erased_in_place(fields) }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn set_in_archetype_by_entity(
        components: &ComponentRegistry,
        archetypes: &mut Archetypes,
        archetype_id: Option<ArchetypeId>,
        entity: Entity,
        fields: ErasedComponents<ErasedField>,
    ) {
        let Some(archetype_id) = archetype_id else {
            unsafe { Self::drop_erased_in_place(components, fields) }
            return;
        };

        let Some(info) = Self::get_info_mut(archetypes, archetype_id) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
        let fields = info.storage_mut().insert_erased(components, entity, fields);
        if let Some(fields) = fields {
            unsafe { Self::drop_erased_in_place(components, fields) }
            unreachable!("duplicated entity {entity:?}")
        }
    }

    #[inline]
    fn move_out_of_archetype_by_entity(
        components: &ComponentRegistry,
        archetypes: &mut Archetypes,
        archetype_id: Option<ArchetypeId>,
        entity: Entity,
    ) -> ErasedComponents<ErasedField> {
        let Some(archetype_id) = archetype_id else {
            return ErasedComponents::with_capacity(1);
        };

        let Some(info) = Self::get_info_mut(archetypes, archetype_id) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
        let Some(fields) = info.storage_mut().remove_erased(components, entity) else {
            unreachable!("{entity:?} should exist in archetype {archetype_id:?}")
        };
        fields
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

        let Some((key, _)) = archetypes.get_index(archetype_id.index()) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
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

        let Some((key, _)) = archetypes.get_index(archetype_id.index()) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
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
            let Some(info) = Self::get_info(archetypes, archetype_id) else {
                unreachable!("archetype {archetype_id:?} should exist")
            };
            if !info.storage().contains(entity) {
                unreachable!("archetype {archetype_id:?} should contain entity {entity:?}");
            }
            return Some(archetype_id);
        }

        archetypes
            .values()
            .position(|info| info.storage().contains(entity))
            .map(ArchetypeId)
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
            return Self::register_from_slice(archetypes, graph, components, &component_ids, key);
        };
        if let &[component_id] = component_ids {
            if let Some(archetype_id) =
                Self::find_archetype_after(graph, archetype_id, component_id)
            {
                return archetype_id;
            }
        }

        let Some(info) = Self::get_info(archetypes, archetype_id) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
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
    }

    #[inline]
    fn register_archetype_without_components(
        graph: &mut Graph,
        archetypes: &mut Archetypes,
        components: &ComponentRegistry,
        archetype_id: ArchetypeId,
        component_ids: &[ComponentId],
    ) -> Option<ArchetypeId> {
        if let &[component_id] = component_ids {
            if let Some(archetype_id) =
                Self::find_archetype_before(graph, archetype_id, component_id)
            {
                return Some(archetype_id);
            }
        }

        let Some(info) = Self::get_info(archetypes, archetype_id) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
        let archetype_component_ids = info.storage().component_ids();
        if archetype_component_ids.len() <= 1 {
            return None;
        }

        let component_ids: ArchetypeKey = component_ids.iter().copied().collect();
        let component_ids: Vec<_> = archetype_component_ids
            .filter(|component_id| !component_ids.contains(component_id))
            .collect();
        let key = component_ids.iter().copied().collect();
        let archetype_id =
            Self::register_from_slice(archetypes, graph, components, &component_ids, key);
        Some(archetype_id)
    }

    #[inline]
    fn find_archetype_before(
        graph: &Graph,
        archetype_id: ArchetypeId,
        component_id: ComponentId,
    ) -> Option<ArchetypeId> {
        graph
            .edges_directed(archetype_id.index().into(), Direction::Incoming)
            .find(|edge| *edge.weight() == component_id)
            .map(|edge| ArchetypeId(edge.source().index()))
    }

    #[inline]
    fn find_archetype_after(
        graph: &Graph,
        archetype_id: ArchetypeId,
        component_id: ComponentId,
    ) -> Option<ArchetypeId> {
        graph
            .edges_directed(archetype_id.index().into(), Direction::Outgoing)
            .find(|edge| *edge.weight() == component_id)
            .map(|edge| ArchetypeId(edge.target().index()))
    }
}

impl Debug for ArchetypeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, graph } = self;

        let config = [
            DotConfig::NodeNoLabel,
            DotConfig::EdgeNoLabel,
            DotConfig::RankDir(RankDir::LR),
        ];
        let node_attrs = |_, (index, _): (NodeIndex<_>, _)| {
            let archetype_id = ArchetypeId(index.index());
            let Some((_, info)) = archetypes.get_index(index.index()) else {
                unreachable!("archetype {archetype_id:?} should exist")
            };
            let component_ids = info.storage().component_ids();
            format!(r#"shape=box label="{archetype_id:?}\n{component_ids:?}" "#)
        };
        let edge_attrs = |_, edge: EdgeReference<'_, _, _>| {
            let component_id = edge.weight();
            format!(r#"label="{component_id:?}" "#)
        };
        let graph = &Dot::with_attr_getters(graph, &config, &edge_attrs, &node_attrs);

        f.debug_struct("ArchetypeRegistry")
            .field("archetypes", archetypes)
            .field("graph", graph)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ArchetypeIds {
    inner: Range<usize>,
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
        let inner = ArchetypeId(start)..ArchetypeId(end);
        write!(f, "{inner:?}")
    }
}

impl Iterator for ArchetypeIds {
    type Item = ArchetypeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(ArchetypeId)
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
        inner.nth(n).map(ArchetypeId)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(ArchetypeId)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(ArchetypeId)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(ArchetypeId)
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
        inner.next_back().map(ArchetypeId)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(ArchetypeId)
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
