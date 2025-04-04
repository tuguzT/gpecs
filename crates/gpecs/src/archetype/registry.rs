use std::{
    collections::BTreeSet,
    fmt::{self, Debug},
};

use indexmap::IndexMap;
use petgraph::{
    dot::{Config as DotConfig, Dot},
    graph::EdgeReference,
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
    graph: Graph<ArchetypeId, ComponentId, Directed, usize>,
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

        let component_ids = B::component_ids(context, components)?.into_iter().collect();
        if let Some(id) = Self::archetype_id_from_inner(archetypes, &component_ids) {
            return Ok(id);
        }

        let storage = ArchetypeStorage::of::<B>(components, context)
            .expect("component ids of this bundle should be unique");
        let id = self.register_inner(component_ids, storage);
        Ok(id)
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

        let component_ids = try_collect_component_ids(component_ids, ArchetypeKey::insert)?;
        if let Some(id) = Self::archetype_id_from_inner(archetypes, &component_ids) {
            return Ok(id);
        }

        let storage = ArchetypeStorage::new(components, component_ids.iter().copied())
            .expect("component ids of this bundle should be unique");
        let id = self.register_inner(component_ids, storage);
        Ok(id)
    }

    #[inline]
    fn register_inner(
        &mut self,
        component_ids: ArchetypeKey,
        storage: ArchetypeStorage,
    ) -> ArchetypeId {
        let Self { archetypes, graph } = self;

        let id = ArchetypeId(archetypes.len());

        let info = ArchetypeInfo { id, storage };
        if let Some(_) = archetypes.insert(component_ids, info) {
            panic!("duplicate archetype registration")
        }

        let node = graph.add_node(id);
        let _ = node; // TODO: store node index somewhere

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

        let component_ids = try_collect_component_ids(component_ids, ArchetypeKey::insert)?;
        let id = Self::archetype_id_from_inner(archetypes, &component_ids);
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

        let component_ids = B::component_ids(context, components)?.into_iter().collect();
        let id = Self::archetype_id_from_inner(archetypes, &component_ids);
        Ok(id)
    }

    #[inline]
    fn archetype_id_from_inner(
        archetypes: &IndexMap<ArchetypeKey, ArchetypeInfo>,
        component_ids: &ArchetypeKey,
    ) -> Option<ArchetypeId> {
        let (index, _, _) = archetypes.get_full(component_ids)?;
        Some(ArchetypeId(index))
    }
}

impl Debug for ArchetypeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, graph } = self;

        let config = [DotConfig::NodeNoLabel, DotConfig::EdgeNoLabel];
        let node_attrs = |_, (_, id)| format!(r#"label="{id:?}""#);
        let edge_attrs = |_, edge: EdgeReference<_, _>| format!(r#"label="{:?}""#, edge.weight());
        let graph = &Dot::with_attr_getters(graph, &config, &edge_attrs, &node_attrs);

        f.debug_struct("ArchetypeRegistry")
            .field("archetypes", archetypes)
            .field("graph", graph)
            .finish()
    }
}
