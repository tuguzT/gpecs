use std::{
    collections::BTreeSet,
    fmt::{self, Debug},
};

use indexmap::IndexMap;
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
        let Self { archetypes, .. } = self;

        let component_ids: Vec<_> = B::component_ids(context, components)?.into_iter().collect();
        let archetype_key = component_ids.iter().copied().collect();
        if let Some(archetype_id) = Self::archetype_id_from_inner(archetypes, &archetype_key) {
            return Ok(archetype_id);
        }

        let storage = ArchetypeStorage::of::<B>(components, context)
            .expect("component ids of this bundle should be unique");
        let archetype_id = self.register_range_to_inclusive(components, component_ids, storage);
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
        let Self { archetypes, .. } = self;

        let component_ids: Vec<_> = component_ids.into_iter().collect();
        let archetype_key = {
            let component_ids = component_ids.iter().copied();
            try_collect_component_ids(component_ids, ArchetypeKey::insert)?
        };
        if let Some(archetype_id) = Self::archetype_id_from_inner(archetypes, &archetype_key) {
            return Ok(archetype_id);
        }

        let storage = ArchetypeStorage::new(components, component_ids.iter().copied())
            .expect("component ids of this bundle should be unique");
        let archetype_id = self.register_range_to_inclusive(components, component_ids, storage);
        Ok(archetype_id)
    }

    #[inline]
    fn register_range_to_inclusive(
        &mut self,
        components: &ComponentRegistry,
        component_ids: Vec<ComponentId>,
        storage: ArchetypeStorage,
    ) -> ArchetypeId {
        let Self { archetypes, graph } = self;

        let count = component_ids.len();
        let mut ids = Vec::with_capacity(count);
        let mut storage = Some(storage);
        for (index, &component_id) in component_ids.iter().enumerate() {
            let component_ids: Vec<_> = component_ids[..=index].iter().copied().collect();
            let archetype_key = component_ids.iter().copied().collect();
            if let Some(archetype_id) = Self::archetype_id_from_inner(archetypes, &archetype_key) {
                ids.push((component_id, archetype_id));
                continue;
            }

            let storage = if index < count - 1 {
                ArchetypeStorage::new(components, component_ids)
                    .expect("component ids of this bundle should be unique")
            } else {
                storage
                    .take()
                    .expect("this should be the last iteration of the loop")
            };
            let archetype_id = Self::register_inner(archetypes, graph, archetype_key, storage);
            ids.push((component_id, archetype_id));
        }

        for ids in ids.windows(2) {
            let [(_, archetype_from), (component_id, archetype_to)] = ids else {
                unreachable!("slice of id pairs should contain two elements")
            };
            let archetype_from_index = archetype_from.index().into();
            let archetype_to_index = archetype_to.index().into();
            let _ = graph.update_edge(archetype_from_index, archetype_to_index, *component_id);
        }

        let (_, archetype_id) = ids
            .pop()
            .expect("input set of component should not be empty");
        archetype_id
    }

    #[inline]
    fn register_inner(
        archetypes: &mut IndexMap<ArchetypeKey, ArchetypeInfo>,
        graph: &mut Graph<(), ComponentId, Directed, usize>,
        archetype_key: ArchetypeKey,
        storage: ArchetypeStorage,
    ) -> ArchetypeId {
        let id = ArchetypeId(archetypes.len());

        let info = ArchetypeInfo { id, storage };
        if let Some(_) = archetypes.insert(archetype_key, info) {
            panic!("duplicate archetype registration")
        }

        let index = id.index();
        let node_index = graph.add_node(()).index();
        assert_eq!(
            index, node_index,
            "archetype index {index} should be equal to node index {node_index}",
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
        let id = Self::archetype_id_from_inner(archetypes, &archetype_key);
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
        let id = Self::archetype_id_from_inner(archetypes, &archetype_key);
        Ok(id)
    }

    #[inline]
    fn archetype_id_from_inner(
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
            format!(r#"label="{archetype_id:?}\n{component_ids:?}" shape=box"#)
        };
        let edge_attrs = |_, edge: EdgeReference<'_, _, _>| {
            let component_id = edge.weight();
            format!(r#"label="{component_id:?}""#)
        };
        let graph = &Dot::with_attr_getters(graph, &config, &edge_attrs, &node_attrs);

        f.debug_struct("ArchetypeRegistry")
            .field("archetypes", archetypes)
            .field("graph", graph)
            .finish()
    }
}
