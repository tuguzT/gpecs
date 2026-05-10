use std::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Deref, DerefMut},
};

use gpecs_archetype::{
    bundle::{Bundle, erased::traits::ErasedArchetypeKind},
    erased::ErasedArchetype,
};
use gpecs_component::registry::traits::{ComponentIdFrom, FromComponentType};
use gpecs_itertools::Itertools as _;
use indexmap::{Equivalent, set::MutableValues};
use itertools::Itertools;
use petgraph::{
    Direction,
    dot::{Config as DotConfig, Dot, RankDir},
    graph::{DiGraph, EdgeReference, NodeIndex},
    visit::{Bfs, EdgeRef, GraphBase, GraphRef, Visitable, Walker, WalkerIter},
};

use crate::{
    archetype::{
        ErasedDropMeta,
        erased::ErasedArchetypeView,
        registry::{
            ArchetypeId, EntityLocation, ErasedArchetypeCow, IterMut,
            error::{
                InsertExactAtErrorKind, InvalidEntityLocationError, InvalidEntityLocationErrorKind,
                RemoveExactAtError,
            },
        },
        storage::ArchetypeStorage,
    },
    bundle::erased::{ErasedBorrowedBundle, ErasedBundleKind},
    component::{
        erased::WithErasedDrop,
        registry::{ComponentId, ComponentRegistryView},
    },
    entity::Entity,
    hash::IndexSet,
    soa::layout::WithLayout,
};

use super::{
    id::{archetype_id_from_usize, archetype_id_into_usize},
    key::ArchetypeKey,
};

pub type Archetypes = IndexSet<ArchetypesItem>;
pub type Graph = DiGraph<(), ComponentId, u32>;

#[repr(transparent)]
pub struct ArchetypesItem {
    storage: ArchetypeStorage,
}

impl ArchetypesItem {
    #[inline]
    fn as_key(&self) -> ArchetypeKey<'_, ErasedDropMeta> {
        let Self { storage } = self;

        let archetype = storage.archetype();
        ArchetypeKey::new(archetype)
    }
}

impl Debug for ArchetypesItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { storage } = self;
        Debug::fmt(storage, f)
    }
}

impl PartialEq for ArchetypesItem {
    fn eq(&self, other: &Self) -> bool {
        let other = &other.as_key();
        self.as_key().eq(other)
    }
}

impl Eq for ArchetypesItem {}

impl PartialOrd for ArchetypesItem {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArchetypesItem {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let other = &other.as_key();
        self.as_key().cmp(other)
    }
}

impl Hash for ArchetypesItem {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_key().hash(state);
    }
}

impl Deref for ArchetypesItem {
    type Target = ArchetypeStorage;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let Self { storage } = self;
        storage
    }
}

impl DerefMut for ArchetypesItem {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self { storage } = self;
        storage
    }
}

impl<Meta> Equivalent<ArchetypesItem> for ArchetypeKey<'_, Meta> {
    #[inline]
    fn equivalent(&self, item: &ArchetypesItem) -> bool {
        item.as_key().eq(self)
    }
}

type GraphWalkerInner<G> = Bfs<<G as GraphBase>::NodeId, <G as Visitable>::Map>;

pub struct GraphWalker<G>
where
    G: GraphRef<NodeId = NodeIndex<u32>> + Visitable,
    GraphWalkerInner<G>: Walker<G, Item = NodeIndex<u32>>,
{
    walker: WalkerIter<GraphWalkerInner<G>, G>,
    start: ArchetypeId,
    exclusive: bool,
}

impl<G> GraphWalker<G>
where
    G: GraphRef<NodeId = NodeIndex<u32>> + Visitable,
    GraphWalkerInner<G>: Walker<G, Item = NodeIndex<u32>>,
{
    pub fn new(graph: G, start: ArchetypeId, exclusive: bool) -> Self {
        let walker = GraphWalkerInner::<G>::new(graph, start.into_u32().into()).iter(graph);
        Self {
            walker,
            start,
            exclusive,
        }
    }

    #[inline]
    pub fn graph(&self) -> G {
        let Self { walker, .. } = self;
        walker.context()
    }

    #[inline]
    pub fn start(&self) -> ArchetypeId {
        let Self { start, .. } = *self;
        start
    }

    #[inline]
    pub fn is_exclusive(&self) -> bool {
        let Self { exclusive, .. } = *self;
        exclusive
    }

    #[inline]
    pub fn is_inclusive(&self) -> bool {
        !self.is_exclusive()
    }
}

impl<G> Clone for GraphWalker<G>
where
    G: GraphRef<NodeId = NodeIndex<u32>> + Visitable<Map: Clone>,
    GraphWalkerInner<G>: Walker<G, Item = NodeIndex<u32>>,
{
    fn clone(&self) -> Self {
        let Self {
            ref walker,
            start,
            exclusive,
        } = *self;

        Self {
            walker: walker.clone(),
            start,
            exclusive,
        }
    }
}

impl<G> Iterator for GraphWalker<G>
where
    G: GraphRef<NodeId = NodeIndex<u32>> + Visitable,
    GraphWalkerInner<G>: Walker<G, Item = NodeIndex<u32>>,
{
    type Item = ArchetypeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut walker,
            start,
            exclusive,
        } = *self;

        let index = if exclusive {
            walker.find(|index| index.index() != archetype_id_into_usize(start))
        } else {
            walker.next()
        }?;

        let archetype_id = archetype_id_from_usize(index.index());
        Some(archetype_id)
    }
}

#[inline]
pub fn find_archetype(
    archetypes: &Archetypes,
    archetype: ErasedArchetypeView<impl Sized>,
) -> Option<ArchetypeId> {
    let key = ArchetypeKey::new(archetype);
    let index = archetypes.get_index_of(&key)?;
    Some(archetype_id_from_usize(index))
}

#[inline]
pub fn get_archetype_storage(
    archetypes: &Archetypes,
    archetype_id: ArchetypeId,
) -> Option<&ArchetypeStorage> {
    let index = archetype_id_into_usize(archetype_id);
    archetypes.get_index(index).map(Deref::deref)
}

#[inline]
#[track_caller]
pub fn unwrap_archetype_storage(
    archetypes: &Archetypes,
    archetype_id: ArchetypeId,
) -> &ArchetypeStorage {
    let Some(storage) = get_archetype_storage(archetypes, archetype_id) else {
        unreachable!("{archetype_id} should exist")
    };
    storage
}

#[inline]
pub fn get_archetype_storage_mut(
    archetypes: &mut Archetypes,
    archetype_id: ArchetypeId,
) -> Option<&mut ArchetypeStorage> {
    let index = archetype_id_into_usize(archetype_id);
    archetypes.get_index_mut2(index).map(DerefMut::deref_mut)
}

#[inline]
#[track_caller]
pub fn unwrap_archetype_storage_mut(
    archetypes: &mut Archetypes,
    archetype_id: ArchetypeId,
) -> &mut ArchetypeStorage {
    let Some(storage) = get_archetype_storage_mut(archetypes, archetype_id) else {
        unreachable!("{archetype_id} should exist")
    };
    storage
}

#[inline]
pub fn get_archetype_storage_pair_mut(
    archetypes: &mut Archetypes,
    a: ArchetypeId,
    b: ArchetypeId,
) -> Option<(&mut ArchetypeStorage, &mut ArchetypeStorage)> {
    let a = archetype_id_into_usize(a);
    let b = archetype_id_into_usize(b);
    IterMut::new(archetypes)
        .get_pair(a, b)
        .map(|((_, a), (_, b))| (a, b))
}

#[inline]
#[track_caller]
pub fn unwrap_archetype_storage_pair_mut(
    archetypes: &mut Archetypes,
    a: ArchetypeId,
    b: ArchetypeId,
) -> (&mut ArchetypeStorage, &mut ArchetypeStorage) {
    let Some(pair) = get_archetype_storage_pair_mut(archetypes, a, b) else {
        unreachable!("{a} and {b} should exist & differ from each other")
    };
    pair
}

#[inline]
pub fn find_location(archetypes: &Archetypes, entity: Entity) -> EntityLocation {
    let index = archetypes
        .iter()
        .position(|info| info.storage.contains(entity));
    let Some(index) = index else {
        return EntityLocation::WithoutComponents;
    };

    let archetype_id = archetype_id_from_usize(index);
    EntityLocation::WithComponents(archetype_id)
}

#[inline]
pub fn check_location(
    archetypes: &Archetypes,
    entity: Entity,
    location: EntityLocation,
) -> Result<(), InvalidEntityLocationError> {
    let EntityLocation::WithComponents(archetype_id) = location else {
        // FIXME: this check is too expensive, especially if done for every entity
        #[cfg(any(debug_assertions, test))]
        if let Some(archetype_id) = find_location(archetypes, entity).archetype_id() {
            let kind = InvalidEntityLocationErrorKind::EntityHasComponents;
            let error = InvalidEntityLocationError::new(entity, archetype_id, kind);
            return Err(error);
        }

        return Ok(());
    };

    let Some(storage) = get_archetype_storage(archetypes, archetype_id) else {
        let kind = InvalidEntityLocationErrorKind::UnknownArchetype;
        let error = InvalidEntityLocationError::new(entity, archetype_id, kind);
        return Err(error);
    };

    if !storage.contains(entity) {
        let kind = InvalidEntityLocationErrorKind::EntityNotFound;
        let error = InvalidEntityLocationError::new(entity, archetype_id, kind);
        return Err(error);
    }

    Ok(())
}

#[inline]
pub fn graph_dot_scoped<F, R>(archetypes: &Archetypes, graph: &Graph, f: F) -> R
where
    F: FnOnce(&Dot<&Graph>) -> R,
{
    let config = [
        DotConfig::NodeNoLabel,
        DotConfig::EdgeNoLabel,
        DotConfig::RankDir(RankDir::LR),
    ];
    let node_attrs = |_, (index, &()): (NodeIndex<_>, _)| {
        let archetype_id = archetype_id_from_usize(index.index());
        let storage = unwrap_archetype_storage(archetypes, archetype_id);
        let component_ids = storage.archetype().into_component_ids();
        format!(r#"shape=box label="{archetype_id:?}\n{component_ids:?}" "#)
    };
    let edge_attrs = |_, edge: EdgeReference<'_, _, _>| {
        let component_id = edge.weight();
        format!(r#"label="{component_id:?}" "#)
    };
    let dot = Dot::with_attr_getters(graph, &config, &edge_attrs, &node_attrs);
    f(&dot)
}

#[inline]
pub fn insert_storage(
    archetypes: &mut Archetypes,
    graph: &mut Graph,
    storage: ArchetypeStorage,
) -> ArchetypeId {
    let index = archetypes.len();
    let id = archetype_id_from_usize(index);

    let item = ArchetypesItem { storage };
    if archetypes.replace(item).is_some() {
        unreachable!("duplicate archetype registration")
    }

    let node_index = graph.add_node(()).index();
    assert_eq!(
        index, node_index,
        "archetype index {index} must be equal to node index {node_index}",
    );

    id
}

#[inline]
pub fn register<M>(
    archetypes: &mut Archetypes,
    graph: &mut Graph,
    components: &ComponentRegistryView<M, impl ?Sized>,
    archetype: ErasedArchetypeCow<ErasedDropMeta>,
) -> ArchetypeId
where
    M: WithLayout + WithErasedDrop,
{
    let archetype_view = archetype.as_view();
    assert!(
        !archetype_view.is_empty(),
        "archetype should contain at least one component",
    );

    if let Some(archetype_id) = find_archetype(archetypes, archetype_view) {
        return archetype_id;
    }

    let before: Vec<_> = register_before(archetypes, graph, components, archetype_view)
        .into_iter()
        .flatten()
        .collect();
    let storage = ArchetypeStorage::from_archetype(archetype.into_owned());
    let archetype_to = insert_storage(archetypes, graph, storage);

    for (archetype_from, component_id) in before {
        let archetype_from = archetype_from.into_u32().into();
        let archetype_to = archetype_to.into_u32().into();
        graph.update_edge(archetype_from, archetype_to, component_id);
    }
    archetype_to
}

#[inline]
pub fn register_before<M>(
    archetypes: &mut Archetypes,
    graph: &mut Graph,
    components: &ComponentRegistryView<M, impl ?Sized>,
    archetype: ErasedArchetypeView<impl Sized>,
) -> Option<impl IntoIterator<Item = (ArchetypeId, ComponentId)>>
where
    M: WithLayout + WithErasedDrop,
{
    #[cold]
    #[inline(never)]
    #[track_caller]
    fn difference_fail(key: ArchetypeKey<impl Sized>, sub_key: ArchetypeKey<impl Sized>) -> ! {
        unreachable!("difference of {key:?} from {sub_key:?} should have exactly one element")
    }

    let len = archetype.len();
    if len <= 1 {
        return None;
    }

    let key = ArchetypeKey::new(archetype);
    let register_subset = move |component_ids| {
        let archetype = ErasedArchetype::new(components, component_ids)
            .expect("components should be unique & registered");

        let sub_key = ArchetypeKey::new(archetype.as_view());
        let Some([component_id]) = key.difference(sub_key).collect_array() else {
            difference_fail(key, sub_key)
        };

        let archetype_id = register(archetypes, graph, components, archetype.into());
        (archetype_id, component_id)
    };
    archetype
        .into_component_ids()
        .combinations(len - 1)
        .map(register_subset)
        .into()
}

#[inline]
pub fn insert_into_archetype<T>(
    archetypes: &mut Archetypes,
    archetype_id: ArchetypeId,
    entity: Entity,
    bundle: ErasedBundleKind<T>,
) where
    T: ErasedArchetypeKind<Meta = ErasedDropMeta>,
{
    assert!(
        !bundle.archetype().is_empty(),
        "bundle should contain at least one component",
    );

    let storage = unwrap_archetype_storage_mut(archetypes, archetype_id);
    if let Err(error) = storage.insert(entity, bundle) {
        unreachable!("failed to insert {entity} into {archetype_id}: {error}")
    }
}

#[inline]
pub fn insert_bundle_into_archetype<B>(
    archetypes: &mut Archetypes,
    components: &ComponentRegistryView<
        impl Sized,
        impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
    >,
    archetype_id: ArchetypeId,
    entity: Entity,
    value: B,
) where
    B: Bundle,
{
    let storage = unwrap_archetype_storage_mut(archetypes, archetype_id);
    if let Err(error) = storage.insert_bundle(components, entity, value) {
        let error = error.into_source();
        unreachable!("failed to insert {entity} into {archetype_id}: {error}")
    }
}

#[cold]
#[track_caller]
#[inline(never)]
pub fn assert_entity_should_exist(entity: Entity, archetype_id: ArchetypeId) -> ! {
    unreachable!("{entity} should exist in {archetype_id}")
}

#[inline]
pub fn remove_from_archetype(
    archetypes: &mut Archetypes,
    archetype_id: ArchetypeId,
    entity: Entity,
) -> ErasedBorrowedBundle<'_, ErasedDropMeta> {
    let storage = unwrap_archetype_storage_mut(archetypes, archetype_id);
    let Some(bundle) = storage.remove(entity) else {
        assert_entity_should_exist(entity, archetype_id)
    };
    bundle
}

#[inline]
pub fn remove_bundle_from_archetype<B>(
    archetypes: &mut Archetypes,
    components: &ComponentRegistryView<
        impl WithLayout + WithErasedDrop,
        impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
    >,
    archetype_id: ArchetypeId,
    entity: Entity,
) -> B
where
    B: Bundle,
{
    let storage = unwrap_archetype_storage_mut(archetypes, archetype_id);
    let bundle = match storage.remove_bundle(components, entity) {
        Ok(bundle) => bundle,
        Err(error) => unreachable!("failed to remove {entity} from {archetype_id}: {error}"),
    };
    let Some(bundle) = bundle else {
        assert_entity_should_exist(entity, archetype_id)
    };
    bundle
}

#[inline]
pub fn destroy_in_archetype(
    archetypes: &mut Archetypes,
    archetype_id: ArchetypeId,
    entity: Entity,
) {
    let storage = unwrap_archetype_storage_mut(archetypes, archetype_id);
    if !storage.destroy(entity) {
        assert_entity_should_exist(entity, archetype_id)
    }
}

#[inline]
pub fn insert_exact_archetypes<M>(
    graph: &mut Graph,
    archetypes: &mut Archetypes,
    components: &ComponentRegistryView<M, impl ?Sized>,
    entity: Entity,
    location: EntityLocation,
    components_to_insert: ErasedArchetypeView<ErasedDropMeta>,
) -> Result<(Option<ArchetypeId>, ArchetypeId), InsertExactAtErrorKind>
where
    M: WithLayout + WithErasedDrop,
{
    check_location(archetypes, entity, location)?;

    let old_archetype = location.into();
    if let Some(archetype_id) = old_archetype {
        let storage = unwrap_archetype_storage(archetypes, archetype_id);
        storage
            .archetype()
            .has_no_components(components_to_insert.as_view())?;
    }

    let new_archetype = register_archetype_with_components(
        graph,
        archetypes,
        components,
        old_archetype,
        components_to_insert,
    );
    Ok((old_archetype, new_archetype))
}

#[inline]
pub fn insert_archetypes<M>(
    graph: &mut Graph,
    archetypes: &mut Archetypes,
    components: &ComponentRegistryView<M, impl ?Sized>,
    entity: Entity,
    location: EntityLocation,
    components_to_insert: ErasedArchetypeView<ErasedDropMeta>,
) -> Result<(Option<ArchetypeId>, ArchetypeId), InvalidEntityLocationError>
where
    M: WithLayout + WithErasedDrop,
{
    check_location(archetypes, entity, location)?;

    let old_archetype = location.into();
    let new_archetype = register_archetype_with_components(
        graph,
        archetypes,
        components,
        old_archetype,
        components_to_insert,
    );
    Ok((old_archetype, new_archetype))
}

#[inline]
pub fn remove_exact_archetypes<M>(
    graph: &mut Graph,
    archetypes: &mut Archetypes,
    components: &ComponentRegistryView<M, impl ?Sized>,
    entity: Entity,
    location: EntityLocation,
    components_to_remove: ErasedArchetypeView<impl Sized>,
) -> Result<Option<(ArchetypeId, Option<ArchetypeId>)>, RemoveExactAtError>
where
    M: WithLayout + WithErasedDrop,
{
    check_location(archetypes, entity, location)?;
    let EntityLocation::WithComponents(old_archetype) = location else {
        return Ok(None);
    };

    let storage = unwrap_archetype_storage(archetypes, old_archetype);
    storage
        .archetype()
        .has_components(components_to_remove.as_view())?;

    let new_archetype = register_archetype_without_components(
        graph,
        archetypes,
        components,
        old_archetype,
        components_to_remove,
    );
    Ok(Some((old_archetype, new_archetype)))
}

#[inline]
pub fn remove_archetypes<M>(
    graph: &mut Graph,
    archetypes: &mut Archetypes,
    components: &ComponentRegistryView<M, impl ?Sized>,
    entity: Entity,
    location: EntityLocation,
    components_to_remove: ErasedArchetypeView<impl Sized>,
) -> Result<Option<(ArchetypeId, Option<ArchetypeId>)>, InvalidEntityLocationError>
where
    M: WithLayout + WithErasedDrop,
{
    check_location(archetypes, entity, location)?;
    let EntityLocation::WithComponents(old_archetype) = location else {
        return Ok(None);
    };

    let new_archetype = register_archetype_without_components(
        graph,
        archetypes,
        components,
        old_archetype,
        components_to_remove,
    );
    Ok(Some((old_archetype, new_archetype)))
}

#[inline]
fn register_archetype_with_components<M>(
    graph: &mut Graph,
    archetypes: &mut Archetypes,
    components: &ComponentRegistryView<M, impl ?Sized>,
    start: Option<ArchetypeId>,
    with_components: ErasedArchetypeView<ErasedDropMeta>,
) -> ArchetypeId
where
    M: WithLayout + WithErasedDrop,
{
    let Some(start) = start else {
        return register(archetypes, graph, components, with_components.into());
    };
    if let Some([component_id]) = with_components.component_ids().collect_array()
        && let Some(archetype_id) = find_archetype_without_component(graph, start, component_id)
    {
        return archetype_id;
    }

    let storage = unwrap_archetype_storage(archetypes, start);
    let component_ids = storage
        .archetype()
        .component_ids()
        .chain(with_components.component_ids())
        .sorted_unstable_by_key(|&component_id| {
            components
                .get_component_descriptor(component_id)
                .map(|info| info.layout().align())
        })
        .unique();
    let archetype = ErasedArchetype::new(components, component_ids)
        .expect("components should be unique & registered");
    register(archetypes, graph, components, archetype.into())
}

#[inline]
fn register_archetype_without_components<M>(
    graph: &mut Graph,
    archetypes: &mut Archetypes,
    components: &ComponentRegistryView<M, impl ?Sized>,
    start: ArchetypeId,
    without_components: ErasedArchetypeView<impl Sized>,
) -> Option<ArchetypeId>
where
    M: WithLayout + WithErasedDrop,
{
    if let Some([component_id]) = without_components.component_ids().collect_array()
        && let Some(archetype_id) = find_archetype_with_component(graph, start, component_id)
    {
        return Some(archetype_id);
    }

    let storage = unwrap_archetype_storage(archetypes, start);
    let archetype_component_ids = storage.archetype().into_component_ids();
    if archetype_component_ids.len() <= 1 {
        return None;
    }

    let component_ids =
        archetype_component_ids.filter(|&component_id| !without_components.contains(component_id));
    let archetype = ErasedArchetype::new(components, component_ids)
        .expect("components should be unique & registered");
    if archetype.is_empty() {
        return None;
    }

    let archetype_id = register(archetypes, graph, components, archetype.into());
    Some(archetype_id)
}

#[inline]
fn find_archetype_with_component(
    graph: &Graph,
    start: ArchetypeId,
    component_id: ComponentId,
) -> Option<ArchetypeId> {
    graph
        .edges_directed(start.into_u32().into(), Direction::Incoming)
        .find(|edge| *edge.weight() == component_id)
        .map(|edge| archetype_id_from_usize(edge.source().index()))
}

#[inline]
fn find_archetype_without_component(
    graph: &Graph,
    start: ArchetypeId,
    component_id: ComponentId,
) -> Option<ArchetypeId> {
    graph
        .edges_directed(start.into_u32().into(), Direction::Outgoing)
        .find(|edge| *edge.weight() == component_id)
        .map(|edge| archetype_id_from_usize(edge.target().index()))
}
