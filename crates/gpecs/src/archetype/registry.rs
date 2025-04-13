use std::{
    borrow::Borrow,
    collections::BTreeSet,
    fmt::{self, Debug},
    iter::{self, FusedIterator},
    ops::Range,
};

use gpecs_soa_erased::field::ErasedField;
use indexmap::IndexMap;
use itertools::Itertools;
use petgraph::{
    dot::{Config as DotConfig, Dot, RankDir},
    graph::{DiGraph, EdgeReference, NodeIndex},
    visit::EdgeRef,
    Direction,
};

use crate::{
    archetype::storage::ArchetypeStorage,
    bundle::{
        error::{DuplicateComponentError, GetComponentsError},
        Bundle,
    },
    component::{
        registry::{ComponentId, ComponentRegistry},
        Component,
    },
    entity::Entity,
};

use super::{erased::ErasedComponents, utils::try_collect_component_ids};

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
pub enum EntityArchetypeStatus {
    Unknown,
    Known(EntityArchetype),
}

impl EntityArchetypeStatus {
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

impl From<EntityArchetype> for EntityArchetypeStatus {
    #[inline]
    fn from(value: EntityArchetype) -> Self {
        EntityArchetypeStatus::Known(value)
    }
}

impl From<Option<EntityArchetype>> for EntityArchetypeStatus {
    #[inline]
    fn from(value: Option<EntityArchetype>) -> Self {
        EntityArchetypeStatus::from_option(value)
    }
}

impl From<EntityArchetypeStatus> for Option<EntityArchetype> {
    #[inline]
    fn from(value: EntityArchetypeStatus) -> Self {
        EntityArchetypeStatus::into_option(value)
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
        context: &B::Context,
    ) -> Result<ArchetypeId, DuplicateComponentError>
    where
        B: Bundle,
    {
        let Self { archetypes, graph } = self;

        let component_ids = B::register_components(context, components)?;
        let component_ids: Vec<_> = component_ids.into_iter().collect();

        let key = component_ids.iter().copied().collect();
        let f = |components| ArchetypeStorage::of::<B>(components, context);
        let archetype_id = Self::register(archetypes, graph, components, &component_ids, key, f);
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_archetype_with_components<I>(
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
    pub fn get_info(&self, id: ArchetypeId) -> Option<&ArchetypeInfo> {
        let Self { archetypes, .. } = self;
        Self::get_archetype_info(archetypes, id)
    }

    #[inline]
    fn get_archetype_info(archetypes: &Archetypes, id: ArchetypeId) -> Option<&ArchetypeInfo> {
        let index = id.index();
        archetypes.get_index(index).map(|(_, info)| info)
    }

    #[inline]
    pub fn get_info_mut(&mut self, id: ArchetypeId) -> Option<&mut ArchetypeInfo> {
        let Self { archetypes, .. } = self;
        Self::get_archetype_info_mut(archetypes, id)
    }

    #[inline]
    fn get_archetype_info_mut(
        archetypes: &mut Archetypes,
        id: ArchetypeId,
    ) -> Option<&mut ArchetypeInfo> {
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
        context: &B::Context,
    ) -> Result<Option<ArchetypeId>, GetComponentsError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;

        let component_ids = B::get_components(context, components)?;
        let key = component_ids.into_iter().collect();
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
    pub fn insert_component<C>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        component: C,
    ) where
        C: Component,
    {
        let archetype_status = EntityArchetypeStatus::Unknown;
        self.insert_component_with(components, entity, archetype_status, component);
    }

    #[inline]
    #[track_caller]
    pub fn insert_component_with<C>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        archetype_status: EntityArchetypeStatus,
        component: C,
    ) -> ArchetypeId
    where
        C: Component,
    {
        let Self { archetypes, graph } = self;

        let component_id = components.register_component::<C>();
        let old_archetype = Self::find_archetype_with_entity_and_without_component(
            archetypes,
            component_id,
            entity,
            archetype_status,
        );
        let new_archetype = Self::register_archetype_with_component(
            graph,
            archetypes,
            components,
            old_archetype,
            component_id,
        );

        let mut old_fields = match old_archetype {
            Some(old_archetype) => {
                let Some(old_info) = Self::get_archetype_info_mut(archetypes, old_archetype) else {
                    unreachable!("archetype {old_archetype:?} should exist")
                };
                let Some(fields) = old_info.storage_mut().remove_erased(components, entity) else {
                    unreachable!("{entity:?} should exist in archetype {old_archetype:?}")
                };
                fields
            }
            None => ErasedComponents::with_capacity(1),
        };

        let field = ErasedField::from::<C>(component);
        if let Some(_) = old_fields.insert(component_id, field) {
            unreachable!("duplicated component {component_id:?}")
        }

        let new_fields = old_fields;
        let Some(new_info) = Self::get_archetype_info_mut(archetypes, new_archetype) else {
            unreachable!("archetype {new_archetype:?} should exist")
        };
        let prev = new_info
            .storage_mut()
            .insert_erased(components, entity, new_fields);
        if let Some(_) = prev {
            unreachable!("duplicated entity {entity:?}")
        }

        new_archetype
    }

    #[inline]
    pub fn remove_component<C>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Option<C>
    where
        C: Component,
    {
        let archetype_status = EntityArchetypeStatus::Unknown;
        let (component, _) = self
            .remove_component_with(components, entity, archetype_status)
            .unzip();
        component
    }

    #[inline]
    #[track_caller]
    pub fn remove_component_with<C>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        archetype_status: EntityArchetypeStatus,
    ) -> Option<(C, EntityArchetype)>
    where
        C: Component,
    {
        let Self { archetypes, graph } = self;

        let component_id = components.register_component::<C>();
        let old_archetype = Self::find_archetype_with_entity_and_with_component(
            archetypes,
            component_id,
            entity,
            archetype_status,
        )?;
        let new_archetype = Self::register_archetype_without_component(
            graph,
            archetypes,
            components,
            old_archetype,
            component_id,
        );

        let Some(old_info) = Self::get_archetype_info_mut(archetypes, old_archetype) else {
            unreachable!("archetype {old_archetype:?} should exist")
        };
        let Some(mut old_fields) = old_info.storage_mut().remove_erased(components, entity) else {
            unreachable!("{entity:?} should exist in archetype {old_archetype:?}")
        };
        let Some(field) = old_fields.swap_remove(&component_id) else {
            unreachable!("component {component_id:?} should exist")
        };

        #[allow(unsafe_code)]
        let Ok(component) = (unsafe { field.into::<C>() }) else {
            unreachable!("field should be convertible to {component_id:?}")
        };
        let new_fields = old_fields;

        if let Some(new_archetype) = new_archetype {
            let Some(new_info) = Self::get_archetype_info_mut(archetypes, new_archetype) else {
                unreachable!("archetype {new_archetype:?} should exist")
            };
            let prev = new_info
                .storage_mut()
                .insert_erased(components, entity, new_fields);
            if let Some(_) = prev {
                unreachable!("duplicated entity {entity:?}")
            }
        }

        Some((component, new_archetype))
    }

    #[inline]
    fn find_archetype_with_entity_and_without_component(
        archetypes: &Archetypes,
        component_id: ComponentId,
        entity: Entity,
        archetype_status: EntityArchetypeStatus,
    ) -> Option<ArchetypeId> {
        if let EntityArchetypeStatus::Known(archetype_id) = archetype_status {
            let archetype_id = archetype_id?;
            let Some((key, info)) = archetypes.get_index(archetype_id.index()) else {
                unreachable!("archetype {archetype_id:?} should exist")
            };
            if !info.storage().contains(entity) {
                unreachable!("archetype {archetype_id:?} should contain entity {entity:?}");
            }
            if key.contains(&component_id) {
                unreachable!("archetype {archetype_id:?} should not contain {component_id:?}");
            }
            return Some(archetype_id);
        }

        archetypes
            .iter()
            .position(|(key, info)| info.storage().contains(entity) && !key.contains(&component_id))
            .map(ArchetypeId)
    }

    #[inline]
    fn find_archetype_with_entity_and_with_component(
        archetypes: &Archetypes,
        component_id: ComponentId,
        entity: Entity,
        archetype_status: EntityArchetypeStatus,
    ) -> Option<ArchetypeId> {
        if let EntityArchetypeStatus::Known(archetype_id) = archetype_status {
            let archetype_id = archetype_id?;
            let Some((key, info)) = archetypes.get_index(archetype_id.index()) else {
                unreachable!("archetype {archetype_id:?} should exist")
            };
            if !info.storage().contains(entity) {
                unreachable!("archetype {archetype_id:?} should contain entity {entity:?}");
            }
            if !key.contains(&component_id) {
                unreachable!("archetype {archetype_id:?} should contain {component_id:?}");
            }
            return Some(archetype_id);
        }

        archetypes
            .iter()
            .position(|(key, info)| info.storage().contains(entity) && key.contains(&component_id))
            .map(ArchetypeId)
    }

    #[inline]
    fn register_archetype_with_component(
        graph: &mut Graph,
        archetypes: &mut Archetypes,
        components: &ComponentRegistry,
        archetype_id: Option<ArchetypeId>,
        component_id: ComponentId,
    ) -> ArchetypeId {
        let Some(archetype_id) = archetype_id else {
            let component_ids = [component_id];
            let key = component_ids.iter().copied().collect();
            return Self::register_from_slice(archetypes, graph, components, &component_ids, key);
        };
        if let Some(archetype_id) = Self::find_archetype_after(graph, archetype_id, component_id) {
            return archetype_id;
        }

        let Some(info) = Self::get_archetype_info(archetypes, archetype_id) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
        let component_ids: Vec<_> = info
            .storage()
            .component_ids()
            .chain(iter::once(component_id))
            .sorted_by_key(|&component_id| {
                components
                    .get_info(component_id)
                    .map(|info| info.descriptor().layout().align())
            })
            .collect();
        let key = component_ids.iter().copied().collect();
        Self::register_from_slice(archetypes, graph, components, &component_ids, key)
    }

    #[inline]
    fn register_archetype_without_component(
        graph: &mut Graph,
        archetypes: &mut Archetypes,
        components: &ComponentRegistry,
        archetype_id: ArchetypeId,
        component_id: ComponentId,
    ) -> Option<ArchetypeId> {
        if let Some(archetype_id) = Self::find_archetype_before(graph, archetype_id, component_id) {
            return Some(archetype_id);
        }

        let Some(info) = Self::get_archetype_info(archetypes, archetype_id) else {
            unreachable!("archetype {archetype_id:?} should exist")
        };
        let component_ids = info.storage().component_ids();
        if component_ids.len() <= 1 {
            return None;
        }

        let component_ids: Vec<_> = component_ids.filter(|&id| id != component_id).collect();
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
