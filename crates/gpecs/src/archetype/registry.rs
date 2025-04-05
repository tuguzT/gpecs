use std::{
    collections::BTreeSet,
    fmt::{self, Debug},
};

use indexmap::IndexMap;
use itertools::Itertools;
use petgraph::{
    dot::{Config as DotConfig, Dot},
    graph::{EdgeReference, NodeIndex},
    Directed, Graph,
};

use crate::{
    bundle::{error::DuplicateComponentError, Bundle},
    component::registry::{ComponentId, ComponentRegistry},
};

use super::{storage::ArchetypeStorage, utils::try_collect_component_ids};

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

#[derive(Default)]
pub struct ArchetypeRegistry {
    archetypes: IndexMap<ArchetypeKey, ArchetypeInfo>,
    graph: Graph<(), ComponentId, Directed, usize>,
}

impl ArchetypeRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            archetypes: IndexMap::new(),
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
        archetypes: &mut IndexMap<ArchetypeKey, ArchetypeInfo>,
        graph: &mut Graph<(), ComponentId, Directed, usize>,
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
        archetypes: &mut IndexMap<ArchetypeKey, ArchetypeInfo>,
        graph: &mut Graph<(), ComponentId, Directed, usize>,
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
        archetypes: &mut IndexMap<ArchetypeKey, ArchetypeInfo>,
        graph: &mut Graph<(), ComponentId, Directed, usize>,
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
        archetypes: &IndexMap<ArchetypeKey, ArchetypeInfo>,
        archetype_key: &ArchetypeKey,
    ) -> Option<ArchetypeId> {
        let (index, _, _) = archetypes.get_full(archetype_key)?;
        Some(ArchetypeId(index))
    }
}

impl Debug for ArchetypeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, graph } = self;

        let config = [DotConfig::NodeNoLabel, DotConfig::EdgeNoLabel];
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
