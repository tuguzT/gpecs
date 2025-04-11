use std::{
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
    bundle::{error::DuplicateComponentError, Bundle},
    component::{
        registry::{ComponentId, ComponentRegistry},
        Component,
    },
    entity::Entity,
};

use super::{
    erased::ErasedComponents, storage::ArchetypeStorage, utils::try_collect_component_ids,
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
pub enum EntityArchetypeStatus {
    Unknown,
    Known(EntityArchetype),
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

        let component_ids: Vec<_> = B::component_ids(context, components)?.into_iter().collect();
        let archetype_key = component_ids.iter().copied().collect();
        if let Some(archetype_id) = Self::find_archetype(archetypes, &archetype_key) {
            return Ok(archetype_id);
        }

        let storage = ArchetypeStorage::of::<B>(components, context)
            .expect("component ids of this bundle should be unique");
        let archetype_id = Self::register(archetypes, graph, components, component_ids, storage);
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
        let archetype_key = {
            let component_ids = component_ids.iter().copied();
            try_collect_component_ids(component_ids, ArchetypeKey::insert)?
        };
        if let Some(archetype_id) = Self::find_archetype(archetypes, &archetype_key) {
            return Ok(archetype_id);
        }

        let storage = ArchetypeStorage::new(components, component_ids.iter().copied())
            .expect("component ids of this bundle should be unique");
        let archetype_id = Self::register(archetypes, graph, components, component_ids, storage);
        Ok(archetype_id)
    }

    #[inline]
    fn register(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        components: &ComponentRegistry,
        component_ids: Vec<ComponentId>,
        storage: ArchetypeStorage,
    ) -> ArchetypeId {
        let archetype_key = component_ids.iter().copied().collect();
        let (before, archetype_to) = match Self::find_archetype(archetypes, &archetype_key) {
            Some(archetype_id) => (Default::default(), archetype_id),
            None => (
                Self::register_before(archetypes, graph, components, component_ids, &archetype_key),
                Self::register_one(archetypes, graph, archetype_key, storage),
            ),
        };

        for (archetype_from, component_id) in before {
            let archetype_from_index = archetype_from.index().into();
            let archetype_to_index = archetype_to.index().into();
            let _ = graph.update_edge(archetype_from_index, archetype_to_index, component_id);
        }
        archetype_to
    }

    #[inline]
    fn register_before(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        components: &ComponentRegistry,
        component_ids: Vec<ComponentId>,
        archetype_key: &ArchetypeKey,
    ) -> Vec<(ArchetypeId, ComponentId)> {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn difference_fail(archetype_key_outer: &ArchetypeKey, archetype_key: &ArchetypeKey) -> ! {
            panic!("difference of {archetype_key_outer:?} from {archetype_key:?} should contain exactly one element")
        }

        let len = component_ids.len();
        if len <= 1 {
            return Default::default();
        }

        let archetype_key_outer = archetype_key;
        component_ids
            .into_iter()
            .combinations(len - 1)
            .map(|component_ids| {
                let archetype_key = component_ids.iter().copied().collect();
                let [component_id] = archetype_key_outer
                    .difference(&archetype_key)
                    .copied()
                    .collect_array()
                    .unwrap_or_else(|| difference_fail(archetype_key_outer, &archetype_key));
                let archetype_id = match Self::find_archetype(archetypes, &archetype_key) {
                    Some(archetype_id) => archetype_id,
                    None => {
                        let storage =
                            ArchetypeStorage::new(components, component_ids.iter().copied())
                                .expect("component ids of this bundle should be unique");
                        Self::register(archetypes, graph, components, component_ids, storage)
                    }
                };
                (archetype_id, component_id)
            })
            .collect()
    }

    #[inline]
    fn register_one(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        archetype_key: ArchetypeKey,
        storage: ArchetypeStorage,
    ) -> ArchetypeId {
        let id = ArchetypeId(archetypes.len());

        let info = ArchetypeInfo { id, storage };
        if let Some(_) = archetypes.insert(archetype_key, info) {
            unreachable!("duplicate archetype registration")
        }

        let index = id.index();
        let node_index = graph.add_node(()).index();
        assert_eq!(
            index, node_index,
            "archetype index {index} must be equal to node index {node_index}",
        );

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

        let index = id.index();
        archetypes.get_index(index).map(|(_, info)| info)
    }

    #[inline]
    pub fn get_info_mut(&mut self, id: ArchetypeId) -> Option<&mut ArchetypeInfo> {
        let Self { archetypes, .. } = self;

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

        let archetype_key = try_collect_component_ids(component_ids, ArchetypeKey::insert)?;
        let id = Self::find_archetype(archetypes, &archetype_key);
        Ok(id)
    }

    #[inline]
    pub fn archetype_id<B>(
        &self,
        components: &mut ComponentRegistry,
        context: &B::Context,
    ) -> Result<Option<ArchetypeId>, DuplicateComponentError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;

        let archetype_key = B::component_ids(context, components)?.into_iter().collect();
        let id = Self::find_archetype(archetypes, &archetype_key);
        Ok(id)
    }

    #[inline]
    fn find_archetype(
        archetypes: &Archetypes,
        archetype_key: &ArchetypeKey,
    ) -> Option<ArchetypeId> {
        let (index, _, _) = archetypes.get_full(archetype_key)?;
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
    ) where
        C: Component,
    {
        let component_id = components.register_component::<C>();

        let old_archetype = match archetype_status {
            EntityArchetypeStatus::Unknown => self
                .archetypes
                .iter()
                .position(|(archetype_key, info)| {
                    !archetype_key.contains(&component_id) && info.storage().contains(entity)
                })
                .map(ArchetypeId),
            EntityArchetypeStatus::Known(archetype_id) => {
                if let Some(archetype_id) = archetype_id {
                    let index = archetype_id.index();
                    let Some((archetype_key, info)) = self.archetypes.get_index(index) else {
                        panic!("archetype {archetype_id:?} should exist")
                    };
                    assert!(
                        !archetype_key.contains(&component_id),
                        "archetype {archetype_id:?} should not contain component {component_id:?}",
                    );
                    assert!(
                        info.storage().contains(entity),
                        "archetype {archetype_id:?} should contain entity {entity:?}",
                    );
                }
                archetype_id
            }
        };

        let (new_archetype, mut old_fields) = match old_archetype {
            Some(old_archetype) => {
                let Some(old_info) = self.get_info_mut(old_archetype) else {
                    unreachable!("old archetype {old_archetype:?} should exist")
                };
                let Some(fields) = old_info.storage_mut().remove_erased(components, entity) else {
                    unreachable!("{entity:?} should exist in old archetype {old_archetype:?}")
                };

                let predicate = |id| id == component_id;
                let graph = &self.graph;
                let new_archetype = Self::find_archetype_after(graph, old_archetype, predicate)
                    .unwrap_or_else(|| {
                        let Some(old_info) = self.get_info(old_archetype) else {
                            unreachable!("old archetype {old_archetype:?} should exist")
                        };
                        let new_component_ids = old_info
                            .storage()
                            .component_ids()
                            .chain(iter::once(component_id))
                            .sorted_by_key(|&component_id| {
                                components
                                    .get_info(component_id)
                                    .map(|info| info.descriptor().layout().align())
                            });
                        self.register_archetype_with_components(components, new_component_ids)
                            .expect("list of new components should contain unique components")
                    });

                (new_archetype, fields)
            }
            None => {
                let fields = ErasedComponents::with_capacity(1);

                let new_component_ids = iter::once(component_id);
                let new_archetype = self
                    .register_archetype_with_components(components, new_component_ids)
                    .expect("list of new components should contain unique components");

                (new_archetype, fields)
            }
        };
        let Some(new_info) = self.get_info_mut(new_archetype) else {
            unreachable!("new archetype {new_archetype:?} should exist")
        };

        let field = ErasedField::from::<C>(component);
        if let Some(_) = old_fields.insert(component_id, field) {
            unreachable!("duplicated component {component_id:?}")
        }

        let new_fields = old_fields;
        let prev = new_info
            .storage_mut()
            .insert_erased(components, entity, new_fields);
        if let Some(_) = prev {
            unreachable!("duplicated entity {entity:?}")
        }
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
        self.remove_component_with(components, entity, archetype_status)
    }

    #[inline]
    #[track_caller]
    #[allow(unsafe_code)]
    pub fn remove_component_with<C>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        archetype_status: EntityArchetypeStatus,
    ) -> Option<C>
    where
        C: Component,
    {
        let component_id = components.register_component::<C>();

        let old_archetype = match archetype_status {
            EntityArchetypeStatus::Unknown => self
                .archetypes
                .iter()
                .position(|(archetype_key, info)| {
                    archetype_key.contains(&component_id) && info.storage().contains(entity)
                })
                .map(ArchetypeId),
            EntityArchetypeStatus::Known(archetype_id) => {
                if let Some(archetype_id) = archetype_id {
                    let index = archetype_id.index();
                    let Some((archetype_key, info)) = self.archetypes.get_index(index) else {
                        panic!("archetype {archetype_id:?} should exist")
                    };
                    assert!(
                        archetype_key.contains(&component_id),
                        "archetype {archetype_id:?} should contain component {component_id:?}",
                    );
                    assert!(
                        info.storage().contains(entity),
                        "archetype {archetype_id:?} should contain entity {entity:?}",
                    );
                }
                archetype_id
            }
        }?;

        let Some(old_info) = self.get_info_mut(old_archetype) else {
            unreachable!("old archetype {old_archetype:?} should exist")
        };

        let Some(mut old_fields) = old_info.storage_mut().remove_erased(components, entity) else {
            unreachable!("{entity:?} should exist in old archetype {old_archetype:?}")
        };
        let Some(field) = old_fields.swap_remove(&component_id) else {
            unreachable!("component {component_id:?} should exist")
        };
        let Ok(component) = (unsafe { field.into::<C>() }) else {
            unreachable!("field should be convertible to {component_id:?}")
        };
        let new_fields = old_fields;

        let predicate = |id| id == component_id;
        let new_archetype = Self::find_archetype_before(&self.graph, old_archetype, predicate)
            .or_else(|| {
                let Some(old_info) = self.get_info(old_archetype) else {
                    unreachable!("old archetype {old_archetype:?} should exist")
                };
                let component_ids = old_info.storage().component_ids();
                if component_ids.len() <= 1 {
                    return None;
                }

                let new_component_ids = component_ids
                    .filter(|&id| id != component_id)
                    .collect::<Vec<_>>();
                let new_archetype = self
                    .register_archetype_with_components(components, new_component_ids)
                    .expect("list of new components should contain unique components");
                Some(new_archetype)
            });

        if let Some(new_archetype) = new_archetype {
            let Some(new_info) = self.get_info_mut(new_archetype) else {
                unreachable!("new archetype {new_archetype:?} should exist")
            };
            let prev = new_info
                .storage_mut()
                .insert_erased(components, entity, new_fields);
            if let Some(_) = prev {
                unreachable!("duplicated entity {entity:?}")
            }
        }

        Some(component)
    }

    #[inline]
    fn find_archetype_before<P>(
        graph: &Graph,
        archetype_id: ArchetypeId,
        mut predicate: P,
    ) -> Option<ArchetypeId>
    where
        P: FnMut(ComponentId) -> bool,
    {
        graph
            .edges_directed(archetype_id.index().into(), Direction::Incoming)
            .find(|edge| predicate(*edge.weight()))
            .map(|edge| ArchetypeId(edge.source().index()))
    }

    #[inline]
    fn find_archetype_after<P>(
        graph: &Graph,
        archetype_id: ArchetypeId,
        mut predicate: P,
    ) -> Option<ArchetypeId>
    where
        P: FnMut(ComponentId) -> bool,
    {
        graph
            .edges_directed(archetype_id.index().into(), Direction::Outgoing)
            .find(|edge| predicate(*edge.weight()))
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
            let index = index.index();
            let archetype_id = ArchetypeId(index);
            let (_, info) = archetypes
                .get_index(index)
                .unwrap_or_else(|| panic!("archetype {archetype_id:?} should exist"));
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
