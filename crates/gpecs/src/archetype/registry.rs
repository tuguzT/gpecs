use std::{
    borrow::Cow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    iter::{self, FusedIterator},
    marker::PhantomData,
    ops::{Deref, DerefMut, Range},
    ptr, slice,
};

pub use gpecs_types::archetype::ArchetypeId;

use indexmap::{Equivalent, set::MutableValues};
use itertools::Itertools;
use petgraph::{
    Direction,
    dot::{Config as DotConfig, Dot, RankDir},
    graph::{DiGraph, EdgeReference, NodeIndex},
    visit::{Bfs, EdgeRef, GraphBase, GraphRef, Reversed, Visitable, Walker, WalkerIter},
};

use crate::{
    archetype::{
        erased::ErasedArchetype,
        error::{
            ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError, InsertBundleError,
            InsertBundleExactError, InvalidEntityLocationError, RemoveBundleExactError,
        },
        storage::{ArchetypeStorage, StorageMeta},
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut,
        erased::{
            ErasedArchetypeKind, ErasedBorrowedBundle, ErasedBundle, ErasedBundleKind, RemovePair,
        },
    },
    component::registry::{ComponentId, ComponentRegistry},
    entity::Entity,
    hash::IndexSet,
    soa::slice::{Iter as SoaIter, IterMut as SoaIterMut, SoaSlices, SoaSlicesMut},
};

/// Location of an entity inside of an archetype registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityLocation {
    /// Entity has components of some archetype attached to it.
    WithComponents(ArchetypeId),
    /// Entity has no components attached to it.
    WithoutComponents,
}

impl EntityLocation {
    /// Returns `true` if entity has components of some archetype attached to it.
    #[inline]
    pub fn has_components(self) -> bool {
        matches!(self, Self::WithComponents(..))
    }

    /// Returns `true` if entity has no components attached to it.
    #[inline]
    pub fn has_no_components(self) -> bool {
        !self.has_components()
    }

    /// Retrieves archetype of some entity if it has components attached to it.
    #[inline]
    pub fn archetype_id(self) -> Option<ArchetypeId> {
        match self {
            Self::WithComponents(archetype_id) => Some(archetype_id),
            Self::WithoutComponents => None,
        }
    }
}

impl From<Option<ArchetypeId>> for EntityLocation {
    #[inline]
    fn from(archetype_id: Option<ArchetypeId>) -> Self {
        match archetype_id {
            Some(archetype_id) => Self::WithComponents(archetype_id),
            None => Self::WithoutComponents,
        }
    }
}

impl From<ArchetypeId> for EntityLocation {
    #[inline]
    fn from(archetype_id: ArchetypeId) -> Self {
        Self::WithComponents(archetype_id)
    }
}

impl From<EntityLocation> for Option<ArchetypeId> {
    #[inline]
    fn from(location: EntityLocation) -> Self {
        location.archetype_id()
    }
}

#[repr(transparent)]
struct ArchetypeKey<Meta> {
    archetype: ErasedArchetype<Meta>,
}

impl<Meta> ArchetypeKey<Meta> {
    #[inline]
    fn from_ref(archetype: &ErasedArchetype<Meta>) -> &Self {
        // SAFETY: Self is `#[repr(transparent)]` over `ErasedArchetype<Meta>`.
        unsafe { &*ptr::from_ref(archetype).cast() }
    }

    #[inline]
    fn len(&self) -> usize {
        let Self { archetype } = self;
        archetype.len()
    }

    #[inline]
    fn contains(&self, component_id: ComponentId) -> bool {
        let Self { archetype } = self;
        archetype.contains(component_id)
    }

    #[inline]
    fn component_ids(&self) -> impl Iterator<Item = ComponentId> {
        let Self { archetype } = self;
        archetype.sorted_iter().map(From::from)
    }

    #[inline]
    fn difference(&self, other: &ArchetypeKey<impl Sized>) -> impl Iterator<Item = ComponentId> {
        self.component_ids().filter(|&id| !other.contains(id))
    }
}

impl<Meta> Debug for ArchetypeKey<Meta> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.component_ids();
        f.debug_set().entries(entries).finish()
    }
}

impl<Meta, OtherMeta> PartialEq<ArchetypeKey<OtherMeta>> for ArchetypeKey<Meta> {
    fn eq(&self, other: &ArchetypeKey<OtherMeta>) -> bool {
        let other = other.component_ids();
        self.component_ids().eq(other)
    }
}

impl<Meta> Eq for ArchetypeKey<Meta> {}

impl<Meta, OtherMeta> PartialOrd<ArchetypeKey<OtherMeta>> for ArchetypeKey<Meta> {
    fn partial_cmp(&self, other: &ArchetypeKey<OtherMeta>) -> Option<cmp::Ordering> {
        let other = other.component_ids();
        self.component_ids().partial_cmp(other)
    }
}

impl<Meta> Ord for ArchetypeKey<Meta> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let other = other.component_ids();
        self.component_ids().cmp(other)
    }
}

impl<Meta> Hash for ArchetypeKey<Meta> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        self.component_ids()
            .for_each(|component_id| component_id.hash(state));
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

#[repr(transparent)]
struct ArchetypesItem {
    info: ArchetypeInfo,
}

impl ArchetypesItem {
    #[inline]
    fn new(info: ArchetypeInfo) -> Self {
        Self { info }
    }

    #[inline]
    fn as_key(&self) -> &ArchetypeKey<StorageMeta> {
        let Self { info } = self;

        let archetype = info.storage.archetype();
        ArchetypeKey::from_ref(archetype)
    }
}

impl Debug for ArchetypesItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { info } = self;
        Debug::fmt(info, f)
    }
}

impl PartialEq for ArchetypesItem {
    fn eq(&self, other: &Self) -> bool {
        let other = other.as_key();
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
        let other = other.as_key();
        self.as_key().cmp(other)
    }
}

impl Hash for ArchetypesItem {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_key().hash(state);
    }
}

impl Deref for ArchetypesItem {
    type Target = ArchetypeInfo;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let Self { info } = self;
        info
    }
}

impl DerefMut for ArchetypesItem {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self { info } = self;
        info
    }
}

impl<Meta> Equivalent<ArchetypesItem> for ArchetypeKey<Meta> {
    #[inline]
    fn equivalent(&self, item: &ArchetypesItem) -> bool {
        item.as_key().eq(self)
    }
}

type Archetypes = IndexSet<ArchetypesItem>;
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
    pub fn register_archetype_of<B>(
        &mut self,
        components: &mut ComponentRegistry,
    ) -> Result<ArchetypeId, DuplicateComponentError>
    where
        B: Bundle,
    {
        let archetype = ErasedArchetype::register::<B>(components)?;
        let archetype_id = self.register_archetype(components, archetype);
        Ok(archetype_id)
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
        let archetype = ErasedArchetype::new(components, component_ids)?;
        let archetype_id = self.register_archetype(components, archetype);
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_archetype<'a>(
        &mut self,
        components: &ComponentRegistry,
        archetype: impl Into<Cow<'a, ErasedArchetype<StorageMeta>>>,
    ) -> ArchetypeId {
        let Self { archetypes, graph } = self;
        Self::register(archetypes, graph, components, archetype.into())
    }

    #[inline]
    fn register(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        components: &ComponentRegistry,
        archetype: Cow<ErasedArchetype<StorageMeta>>,
    ) -> ArchetypeId {
        let archetype_ref = archetype.as_ref();
        assert!(
            !archetype_ref.is_empty(),
            "archetype should contain at least one component",
        );

        if let Some(archetype_id) = find_archetype(archetypes, archetype_ref) {
            return archetype_id;
        }

        let before: Vec<_> = Self::register_before(archetypes, graph, components, archetype_ref)
            .into_iter()
            .flatten()
            .collect();
        let storage = ArchetypeStorage::from_archetype(archetype.into_owned());
        let archetype_to = Self::insert_storage(archetypes, graph, storage);

        for (archetype_from, component_id) in before {
            let archetype_from = archetype_from.into_u32().into();
            let archetype_to = archetype_to.into_u32().into();
            graph.update_edge(archetype_from, archetype_to, component_id);
        }
        archetype_to
    }

    #[inline]
    fn register_before(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        components: &ComponentRegistry,
        archetype: &ErasedArchetype<impl Sized>,
    ) -> Option<impl IntoIterator<Item = (ArchetypeId, ComponentId)>> {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn difference_fail(
            key: &ArchetypeKey<impl Sized>,
            sub_key: &ArchetypeKey<impl Sized>,
        ) -> ! {
            unreachable!("difference of {key:?} from {sub_key:?} should have exactly one element")
        }

        let len = archetype.len();
        if len <= 1 {
            return None;
        }

        let key = ArchetypeKey::from_ref(archetype);
        let register_subset = |component_ids| {
            let archetype = ErasedArchetype::new(components, component_ids)
                .expect("components should be unique & registered");

            let sub_key = ArchetypeKey::from_ref(&archetype);
            let Some([component_id]) = key.difference(sub_key).collect_array() else {
                difference_fail(key, sub_key)
            };

            let archetype_id = Self::register(archetypes, graph, components, archetype.into());
            (archetype_id, component_id)
        };
        archetype
            .component_ids()
            .combinations(len - 1)
            .map(register_subset)
            .into()
    }

    #[inline]
    fn insert_storage(
        archetypes: &mut Archetypes,
        graph: &mut Graph,
        storage: ArchetypeStorage,
    ) -> ArchetypeId {
        let index = archetypes.len();
        let id = archetype_id_from_usize(index);

        let info = ArchetypeInfo { id, storage };
        let item = ArchetypesItem::new(info);
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
        components: &ComponentRegistry,
        component_ids: I,
    ) -> Result<Option<ArchetypeId>, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = ErasedArchetype::<()>::new(components, component_ids)?;
        let archetype_id = self.archetype_id(&archetype);
        Ok(archetype_id)
    }

    #[inline]
    pub fn archetype_id_of<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<Option<ArchetypeId>, ArchetypeError>
    where
        B: Bundle,
    {
        let archetype = ErasedArchetype::<()>::of::<B>(components)?;
        let archetype_id = self.archetype_id(&archetype);
        Ok(archetype_id)
    }

    #[inline]
    pub fn archetype_id(&self, archetype: &ErasedArchetype<impl Sized>) -> Option<ArchetypeId> {
        let Self { archetypes, .. } = self;
        find_archetype(archetypes, archetype)
    }

    #[inline]
    pub fn archetype_ids(&self) -> ArchetypeIds {
        let len = self.len();
        let len = archetype_id_from_usize(len).into_u32();
        ArchetypeIds { inner: 0..len }
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

    // TODO: get erased bundle

    #[inline]
    pub fn find_location(&self, entity: Entity) -> EntityLocation {
        let Self { archetypes, .. } = self;
        find_location(archetypes, entity)
    }

    #[inline]
    pub fn get_bundle<B>(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<BundleRefs<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let location = self.find_location(entity);
        self.get_bundle_at::<B>(components, entity, location)
    }

    #[inline]
    #[track_caller] // TODO: return an error instead
    pub fn get_bundle_at<B>(
        &self,
        components: &ComponentRegistry,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<Option<BundleRefs<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;
        let location = Self::check_location(archetypes, entity, location)
            .expect("entity location should be valid");
        let EntityLocation::WithComponents(archetype_id) = location else {
            return Ok(None);
        };

        let info = unwrap_archetype_info(archetypes, archetype_id);
        let refs = info.storage().get_bundle::<B>(components, entity)?;
        Ok(refs)
    }

    #[inline]
    pub fn get_bundle_mut<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<BundleRefsMut<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let location = self.find_location(entity);
        self.get_bundle_mut_at::<B>(components, entity, location)
    }

    #[inline]
    #[track_caller] // TODO: return an error instead
    pub fn get_bundle_mut_at<B>(
        &mut self,
        components: &ComponentRegistry,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<Option<BundleRefsMut<'_, B>>, IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, .. } = self;
        let location = Self::check_location(archetypes, entity, location)
            .expect("entity location should be valid");
        let EntityLocation::WithComponents(archetype_id) = location else {
            return Ok(None);
        };

        let info = unwrap_archetype_info_mut(archetypes, archetype_id);
        let refs = info.storage.get_bundle_mut::<B>(components, entity)?;
        Ok(refs)
    }

    // TODO: get erased bundles

    #[inline]
    pub fn bundles<'ctx, B>(
        &self,
        components: &'ctx ComponentRegistry,
    ) -> Result<Bundles<'_, 'ctx, B>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, graph } = self;
        Bundles::new(archetypes, graph, components)
    }

    #[inline]
    pub fn bundles_mut<'ctx, B>(
        &mut self,
        components: &'ctx ComponentRegistry,
    ) -> Result<BundlesMut<'_, 'ctx, B>, ArchetypeError>
    where
        B: Bundle,
    {
        let Self { archetypes, graph } = self;
        BundlesMut::new(archetypes, graph, components)
    }

    #[inline]
    pub fn compatible_archetypes_from<I>(
        &self,
        components: &ComponentRegistry,
        component_ids: I,
    ) -> Result<CompatibleArchetypes<'_>, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = ErasedArchetype::<()>::new(components, component_ids)?;
        let archetypes = self.compatible_archetypes(&archetype);
        Ok(archetypes)
    }

    #[inline]
    pub fn compatible_archetypes_of<B>(
        &self,
        components: &ComponentRegistry,
    ) -> Result<CompatibleArchetypes<'_>, ArchetypeError>
    where
        B: Bundle,
    {
        let archetype = ErasedArchetype::<()>::of::<B>(components)?;
        let archetypes = self.compatible_archetypes(&archetype);
        Ok(archetypes)
    }

    #[inline]
    pub fn compatible_archetypes(
        &self,
        archetype: &ErasedArchetype<impl Sized>,
    ) -> CompatibleArchetypes<'_> {
        let Self { archetypes, graph } = self;
        CompatibleArchetypes::new(archetypes, graph, archetype)
    }

    #[inline]
    pub unsafe fn compatible_archetypes_mut_from<I>(
        &mut self,
        components: &ComponentRegistry,
        component_ids: I,
    ) -> Result<CompatibleArchetypesMut<'_>, ArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let archetype = ErasedArchetype::<()>::new(components, component_ids)?;
        let archetypes = self.compatible_archetypes_mut(&archetype);
        Ok(archetypes)
    }

    #[inline]
    pub unsafe fn compatible_archetypes_mut_of<B>(
        &mut self,
        components: &ComponentRegistry,
    ) -> Result<CompatibleArchetypesMut<'_>, ArchetypeError>
    where
        B: Bundle,
    {
        let archetype = ErasedArchetype::<()>::of::<B>(components)?;
        let archetypes = self.compatible_archetypes_mut(&archetype);
        Ok(archetypes)
    }

    #[inline]
    pub fn compatible_archetypes_mut(
        &mut self,
        archetype: &ErasedArchetype<impl Sized>,
    ) -> CompatibleArchetypesMut<'_> {
        let Self { archetypes, graph } = self;
        CompatibleArchetypesMut::new(archetypes, graph, archetype)
    }

    // TODO: insert erased bundle

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
        let location = self.find_location(entity);
        self.insert_bundle_exact_at::<B>(components, entity, value, location)?;
        Ok(())
    }

    #[inline]
    #[track_caller] // TODO: return an error instead
    pub fn insert_bundle_exact_at<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        value: B,
        location: EntityLocation,
    ) -> Result<ArchetypeId, InsertBundleExactError<B>>
    where
        B: Bundle,
    {
        let bundle_components = match ErasedArchetype::register::<B>(components) {
            Ok(archetype) => archetype,
            Err(error) => {
                let reason = error.into();
                return Err(InsertBundleExactError { value, reason });
            }
        };

        let Self { archetypes, graph } = self;
        let old_archetype = Self::check_location(archetypes, entity, location)
            .expect("entity location should be valid")
            .into();

        if let Some(archetype_id) = old_archetype {
            let check_result = unwrap_archetype_info(archetypes, archetype_id)
                .storage()
                .archetype()
                .has_no_components(&bundle_components);
            if let Err(error) = check_result {
                let reason = error.into();
                return Err(InsertBundleExactError { value, reason });
            }
        }

        let new_archetype = Self::register_archetype_with_components(
            graph,
            archetypes,
            components,
            old_archetype,
            &bundle_components,
        );

        let Some(old_archetype) = old_archetype else {
            let info = unwrap_archetype_info_mut(archetypes, new_archetype);
            if let Err(error) = info.storage.insert_bundle(components, entity, value) {
                let error = error.reason;
                unreachable!("failed to insert {entity} into {new_archetype}: {error}")
            }
            return Ok(new_archetype);
        };

        let to_insert = ErasedBundle::try_from(components, value)
            .map_err(|error| error.reason)
            .expect("bundle compatibility should have been already checked");
        let bundle = Self::move_out_of_archetype_by_entity(archetypes, old_archetype, entity)
            .insert(to_insert)
            .expect("old archetype should not have components of the inserted bundle");
        Self::set_in_archetype_by_entity(archetypes, new_archetype, entity, bundle);

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
        let location = self.find_location(entity);
        self.insert_bundle_at::<B>(components, entity, value, location)?;
        Ok(())
    }

    #[inline]
    #[track_caller] // TODO: return an error instead
    pub fn insert_bundle_at<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        value: B,
        location: EntityLocation,
    ) -> Result<ArchetypeId, InsertBundleError<B>>
    where
        B: Bundle,
    {
        let bundle_components = match ErasedArchetype::register::<B>(components) {
            Ok(archetype) => archetype,
            Err(reason) => return Err(InsertBundleError { value, reason }),
        };

        let Self { archetypes, graph } = self;
        let old_archetype = Self::check_location(archetypes, entity, location)
            .expect("entity location should be valid")
            .into();

        let new_archetype = Self::register_archetype_with_components(
            graph,
            archetypes,
            components,
            old_archetype,
            &bundle_components,
        );
        let Some(old_archetype) = old_archetype else {
            let info = unwrap_archetype_info_mut(archetypes, new_archetype);
            if let Err(error) = info.storage.insert_bundle(components, entity, value) {
                let error = error.reason;
                unreachable!("failed to insert {entity} into {new_archetype}: {error}")
            }
            return Ok(new_archetype);
        };

        let to_replace = ErasedBundle::try_from(components, value)
            .map_err(|error| error.reason)
            .expect("bundle compatibility should have been already checked");
        let bundle = Self::move_out_of_archetype_by_entity(archetypes, old_archetype, entity)
            .replace(to_replace)
            .expect("combined bundle should be created successfully");
        Self::set_in_archetype_by_entity(archetypes, new_archetype, entity, bundle);

        Ok(new_archetype)
    }

    // TODO: remove erased bundle

    #[inline]
    pub fn remove_bundle_exact<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
    ) -> Result<Option<B>, RemoveBundleExactError>
    where
        B: Bundle,
    {
        let location = self.find_location(entity);
        let (value, _) = self.remove_bundle_exact_at::<B>(components, entity, location)?;
        Ok(value)
    }

    #[inline]
    #[track_caller] // TODO: return an error instead
    pub fn remove_bundle_exact_at<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<(Option<B>, EntityLocation), RemoveBundleExactError>
    where
        B: Bundle,
    {
        let bundle_components = ErasedArchetype::register::<B>(components)?;

        let Self { archetypes, graph } = self;
        let old_archetype = Self::check_location(archetypes, entity, location)
            .expect("entity location should be valid")
            .into();

        let Some(old_archetype) = old_archetype else {
            return Ok((None, EntityLocation::WithoutComponents));
        };
        unwrap_archetype_info(archetypes, old_archetype)
            .storage()
            .archetype()
            .has_components(&bundle_components)?;

        let new_archetype = Self::register_archetype_without_components(
            graph,
            archetypes,
            components,
            old_archetype,
            &bundle_components,
        );
        let Some(new_archetype) = new_archetype else {
            let info = unwrap_archetype_info_mut(archetypes, old_archetype);
            let value = info
                .storage
                .remove_bundle::<B>(components, entity)
                .expect("archetype should be compatible")
                .expect("storage should contain data of given entity");
            return Ok((Some(value), EntityLocation::WithoutComponents));
        };

        let RemovePair {
            retained: bundle,
            removed,
        } = Self::move_out_of_archetype_by_entity(archetypes, old_archetype, entity)
            .remove(bundle_components)
            .expect("all the bundle components should be present in the old archetype");
        Self::set_in_archetype_by_entity(archetypes, new_archetype, entity, bundle);

        let value = removed
            .downcast(components)
            .expect("archetype should be compatible");
        let location = EntityLocation::WithComponents(new_archetype);
        Ok((Some(value), location))
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
        let location = self.find_location(entity);
        self.remove_bundle_at::<B>(components, entity, location)?;
        Ok(())
    }

    #[inline]
    #[track_caller] // TODO: return an error instead
    pub fn remove_bundle_at<B>(
        &mut self,
        components: &mut ComponentRegistry,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<EntityLocation, DuplicateComponentError>
    where
        B: Bundle,
    {
        let bundle_components = ErasedArchetype::<()>::register::<B>(components)?;

        let Self { archetypes, graph } = self;
        let location = Self::check_location(archetypes, entity, location)
            .expect("entity location should be valid");
        let EntityLocation::WithComponents(old_archetype) = location else {
            return Ok(EntityLocation::WithoutComponents);
        };

        let new_archetype = Self::register_archetype_without_components(
            graph,
            archetypes,
            components,
            old_archetype,
            &bundle_components,
        );
        let Some(new_archetype) = new_archetype else {
            let info = unwrap_archetype_info_mut(archetypes, old_archetype);
            if !info.storage.destroy(entity) {
                unreachable!("{entity} should exist in {old_archetype}")
            }
            return Ok(EntityLocation::WithoutComponents);
        };

        let bundle = Self::move_out_of_archetype_by_entity(archetypes, old_archetype, entity)
            .destroy(&bundle_components)
            .expect("all the bundle components should be present in the old archetype");
        Self::set_in_archetype_by_entity(archetypes, new_archetype, entity, bundle);

        let location = EntityLocation::WithComponents(new_archetype);
        Ok(location)
    }

    #[inline]
    pub fn destroy(&mut self, entity: Entity) -> bool {
        let location = self.find_location(entity);
        self.destroy_at(entity, location)
    }

    #[inline]
    #[track_caller] // TODO: return an error instead
    pub fn destroy_at(&mut self, entity: Entity, location: EntityLocation) -> bool {
        let Self { archetypes, .. } = self;

        let location = Self::check_location(archetypes, entity, location)
            .expect("entity location should be valid");
        let EntityLocation::WithComponents(archetype_id) = location else {
            return false;
        };

        let info = unwrap_archetype_info_mut(archetypes, archetype_id);
        if !info.storage.destroy(entity) {
            unreachable!("{entity} should exist in {archetype_id}")
        }
        true
    }

    #[inline]
    pub fn destroy_all(&mut self) {
        let archetype_ids = self.archetype_ids();
        let Self { archetypes, .. } = self;

        for archetype_id in archetype_ids {
            let info = unwrap_archetype_info_mut(archetypes, archetype_id);
            info.storage.destroy_all();
        }
    }

    #[inline]
    fn set_in_archetype_by_entity<T>(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        entity: Entity,
        bundle: ErasedBundleKind<T>,
    ) where
        T: ErasedArchetypeKind<Meta = StorageMeta>,
    {
        assert!(
            !bundle.archetype().is_empty(),
            "bundle should contain at least one component",
        );

        let info = unwrap_archetype_info_mut(archetypes, archetype_id);
        if let Err(error) = info.storage.insert(entity, bundle) {
            unreachable!("failed to insert {entity} into {archetype_id}: {error}")
        }
    }

    #[inline]
    fn move_out_of_archetype_by_entity(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        entity: Entity,
    ) -> ErasedBorrowedBundle<'_, StorageMeta> {
        let info = unwrap_archetype_info_mut(archetypes, archetype_id);
        let Some(bundle) = info.storage.remove(entity) else {
            unreachable!("{entity} should exist in {archetype_id}")
        };
        bundle
    }

    #[inline]
    fn check_location(
        archetypes: &Archetypes,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<EntityLocation, InvalidEntityLocationError> {
        let EntityLocation::WithComponents(archetype_id) = location else {
            // FIXME: this check is too expensive, especially if done for every entity
            #[cfg(debug_assertions)]
            if let Some(archetype_id) = find_location(archetypes, entity).archetype_id() {
                let error = InvalidEntityLocationError::new(entity, archetype_id, true);
                return Err(error);
            }

            return Ok(location);
        };

        let info = unwrap_archetype_info(archetypes, archetype_id);
        if !info.storage().contains(entity) {
            let error = InvalidEntityLocationError::new(entity, archetype_id, false);
            return Err(error);
        }
        Ok(location)
    }

    #[inline]
    fn register_archetype_with_components(
        graph: &mut Graph,
        archetypes: &mut Archetypes,
        components: &ComponentRegistry,
        start: Option<ArchetypeId>,
        with_components: &ErasedArchetype<StorageMeta>,
    ) -> ArchetypeId {
        let Some(start) = start else {
            return Self::register(archetypes, graph, components, with_components.into());
        };
        if with_components.len() == 1
            && let Some(component_id) = with_components.component_ids().next()
            && let Some(archetype_id) =
                Self::find_archetype_without_component(graph, start, component_id)
        {
            return archetype_id;
        }

        let info = unwrap_archetype_info(archetypes, start);
        let component_ids = info
            .storage()
            .archetype()
            .component_ids()
            .chain(with_components.component_ids())
            .sorted_unstable_by_key(|&component_id| {
                components
                    .get_component_info(component_id)
                    .map(|info| info.descriptor().layout().align())
            })
            .unique();
        let archetype = ErasedArchetype::new(components, component_ids)
            .expect("components should be unique & registered");
        Self::register(archetypes, graph, components, archetype.into())
    }

    #[inline]
    fn register_archetype_without_components(
        graph: &mut Graph,
        archetypes: &mut Archetypes,
        components: &ComponentRegistry,
        start: ArchetypeId,
        without_components: &ErasedArchetype<impl Sized>,
    ) -> Option<ArchetypeId> {
        if without_components.len() == 1
            && let Some(component_id) = without_components.component_ids().next()
            && let Some(archetype_id) =
                Self::find_archetype_with_component(graph, start, component_id)
        {
            return Some(archetype_id);
        }

        let info = unwrap_archetype_info(archetypes, start);
        let archetype_component_ids = info.storage().archetype().component_ids();
        if archetype_component_ids.len() <= 1 {
            return None;
        }

        let component_ids = archetype_component_ids
            .filter(|&component_id| !without_components.contains(component_id));
        let archetype = ErasedArchetype::new(components, component_ids)
            .expect("components should be unique & registered");
        if archetype.is_empty() {
            return None;
        }

        let archetype_id = Self::register(archetypes, graph, components, archetype.into());
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
        let component_ids = info.storage().archetype().component_ids();
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

type GraphWalker<G> = Bfs<<G as GraphBase>::NodeId, <G as Visitable>::Map>;

struct ArchetypeWalker<G>
where
    G: GraphRef<NodeId = NodeIndex<u32>> + Visitable,
    GraphWalker<G>: Walker<G, Item = NodeIndex<u32>>,
{
    walker: WalkerIter<GraphWalker<G>, G>,
    start: ArchetypeId,
    exclusive: bool,
}

impl<G> ArchetypeWalker<G>
where
    G: GraphRef<NodeId = NodeIndex<u32>> + Visitable,
    GraphWalker<G>: Walker<G, Item = NodeIndex<u32>>,
{
    fn new(graph: G, start: ArchetypeId, exclusive: bool) -> Self {
        let walker = GraphWalker::<G>::new(graph, start.into_u32().into()).iter(graph);
        Self {
            walker,
            start,
            exclusive,
        }
    }

    #[inline]
    fn graph(&self) -> G {
        let Self { walker, .. } = self;
        walker.context()
    }

    #[inline]
    fn start(&self) -> ArchetypeId {
        let Self { start, .. } = *self;
        start
    }

    #[inline]
    fn is_exclusive(&self) -> bool {
        let Self { exclusive, .. } = *self;
        exclusive
    }

    #[inline]
    fn is_inclusive(&self) -> bool {
        !self.is_exclusive()
    }
}

impl<G> Clone for ArchetypeWalker<G>
where
    G: GraphRef<NodeId = NodeIndex<u32>> + Visitable<Map: Clone>,
    GraphWalker<G>: Walker<G, Item = NodeIndex<u32>>,
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

impl<G> Iterator for ArchetypeWalker<G>
where
    G: GraphRef<NodeId = NodeIndex<u32>> + Visitable,
    GraphWalker<G>: Walker<G, Item = NodeIndex<u32>>,
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

#[derive(Clone)]
pub struct ArchetypesBefore<'a> {
    archetypes: &'a Archetypes,
    walker: ArchetypeWalker<Reversed<&'a Graph>>,
}

impl<'a> ArchetypesBefore<'a> {
    #[inline]
    fn new(
        archetypes: &'a Archetypes,
        graph: &'a Graph,
        start: ArchetypeId,
        exclusive: bool,
    ) -> Option<Self> {
        let _ = get_archetype_info(archetypes, start)?;
        let graph = Reversed(graph);
        let walker = ArchetypeWalker::new(graph, start, exclusive);

        let me = Self { archetypes, walker };
        Some(me)
    }

    #[inline]
    pub fn start(&self) -> ArchetypeId {
        let Self { walker, .. } = self;
        walker.start()
    }

    #[inline]
    pub fn is_exclusive(&self) -> bool {
        let Self { walker, .. } = self;
        walker.is_exclusive()
    }

    #[inline]
    pub fn is_inclusive(&self) -> bool {
        let Self { walker, .. } = self;
        walker.is_inclusive()
    }
}

impl Debug for ArchetypesBefore<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, walker } = self;

        graph_dot_scoped(archetypes, walker.graph().0, |graph| {
            f.debug_struct("ArchetypesBefore")
                .field("archetypes", archetypes)
                .field("graph", graph)
                .field("start", &walker.start())
                .field("inclusive", &walker.is_inclusive())
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
        } = *self;

        let archetype_id = walker.next()?;
        let info = unwrap_archetype_info(archetypes, archetype_id);
        Some(info)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { archetypes, walker } = self;

        let skip_count = usize::from(walker.is_exclusive());
        let upper = archetypes.len().saturating_sub(skip_count);
        (0, Some(upper))
    }
}

pub struct ArchetypesBeforeMut<'a> {
    archetypes: &'a mut Archetypes,
    walker: ArchetypeWalker<Reversed<&'a Graph>>,
}

impl<'a> ArchetypesBeforeMut<'a> {
    #[inline]
    fn new(
        archetypes: &'a mut Archetypes,
        graph: &'a Graph,
        start: ArchetypeId,
        exclusive: bool,
    ) -> Option<Self> {
        let _ = get_archetype_info(archetypes, start)?;
        let graph = Reversed(graph);
        let walker = ArchetypeWalker::new(graph, start, exclusive);

        let me = Self { archetypes, walker };
        Some(me)
    }

    #[inline]
    pub fn start(&self) -> ArchetypeId {
        let Self { walker, .. } = self;
        walker.start()
    }

    #[inline]
    pub fn is_exclusive(&self) -> bool {
        let Self { walker, .. } = self;
        walker.is_exclusive()
    }

    #[inline]
    pub fn is_inclusive(&self) -> bool {
        let Self { walker, .. } = self;
        walker.is_inclusive()
    }
}

impl Debug for ArchetypesBeforeMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, walker } = self;

        graph_dot_scoped(archetypes, walker.graph().0, |graph| {
            f.debug_struct("ArchetypesBeforeMut")
                .field("archetypes", archetypes)
                .field("graph", graph)
                .field("start", &walker.start())
                .field("inclusive", &walker.is_inclusive())
                .finish()
        })
    }
}

impl<'a> Iterator for ArchetypesBeforeMut<'a> {
    type Item = &'a mut ArchetypeInfo;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { walker, archetypes } = self;

        let archetype_id = walker.next()?;
        let info = unwrap_archetype_info_mut(archetypes, archetype_id);

        // SAFETY: BFS walker is non-recursive, so it must not yield the same node twice
        let info = unsafe { &mut *ptr::from_mut(info) };
        Some(info)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { archetypes, walker } = self;

        let skip_count = usize::from(walker.is_exclusive());
        let upper = archetypes.len().saturating_sub(skip_count);
        (0, Some(upper))
    }
}

#[derive(Clone)]
pub struct ArchetypesAfter<'a> {
    archetypes: &'a Archetypes,
    walker: ArchetypeWalker<&'a Graph>,
}

impl<'a> ArchetypesAfter<'a> {
    #[inline]
    fn new(
        archetypes: &'a Archetypes,
        graph: &'a Graph,
        start: ArchetypeId,
        exclusive: bool,
    ) -> Option<Self> {
        let _ = get_archetype_info(archetypes, start)?;
        let walker = ArchetypeWalker::new(graph, start, exclusive);

        let me = Self { archetypes, walker };
        Some(me)
    }

    #[inline]
    pub fn start(&self) -> ArchetypeId {
        let Self { walker, .. } = self;
        walker.start()
    }

    #[inline]
    pub fn is_exclusive(&self) -> bool {
        let Self { walker, .. } = self;
        walker.is_exclusive()
    }

    #[inline]
    pub fn is_inclusive(&self) -> bool {
        let Self { walker, .. } = self;
        walker.is_inclusive()
    }
}

impl Debug for ArchetypesAfter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, walker } = self;

        graph_dot_scoped(archetypes, walker.graph(), |graph| {
            f.debug_struct("ArchetypesAfter")
                .field("archetypes", archetypes)
                .field("graph", graph)
                .field("start", &walker.start())
                .field("inclusive", &walker.is_inclusive())
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
        } = *self;

        let archetype_id = walker.next()?;
        let info = unwrap_archetype_info(archetypes, archetype_id);
        Some(info)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { archetypes, walker } = self;

        let skip_count = usize::from(walker.is_exclusive());
        let upper = archetypes.len().saturating_sub(skip_count);
        (0, Some(upper))
    }
}

pub struct ArchetypesAfterMut<'a> {
    archetypes: &'a mut Archetypes,
    walker: ArchetypeWalker<&'a Graph>,
}

impl<'a> ArchetypesAfterMut<'a> {
    #[inline]
    fn new(
        archetypes: &'a mut Archetypes,
        graph: &'a Graph,
        start: ArchetypeId,
        exclusive: bool,
    ) -> Option<Self> {
        let _ = get_archetype_info(archetypes, start)?;
        let walker = ArchetypeWalker::new(graph, start, exclusive);

        let me = Self { archetypes, walker };
        Some(me)
    }

    #[inline]
    pub fn start(&self) -> ArchetypeId {
        let Self { walker, .. } = self;
        walker.start()
    }

    #[inline]
    pub fn is_exclusive(&self) -> bool {
        let Self { walker, .. } = self;
        walker.is_exclusive()
    }

    #[inline]
    pub fn is_inclusive(&self) -> bool {
        let Self { walker, .. } = self;
        walker.is_inclusive()
    }
}

impl Debug for ArchetypesAfterMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, walker } = self;

        graph_dot_scoped(archetypes, walker.graph(), |graph| {
            f.debug_struct("ArchetypesAfterMut")
                .field("archetypes", archetypes)
                .field("graph", graph)
                .field("start", &walker.start())
                .field("inclusive", &walker.is_inclusive())
                .finish()
        })
    }
}

impl<'a> Iterator for ArchetypesAfterMut<'a> {
    type Item = &'a mut ArchetypeInfo;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { walker, archetypes } = self;

        let archetype_id = walker.next()?;
        let info = unwrap_archetype_info_mut(archetypes, archetype_id);

        // SAFETY: BFS walker is non-recursive, so it must not yield the same node twice
        let info = unsafe { &mut *ptr::from_mut(info) };
        Some(info)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { archetypes, walker } = self;

        let skip_count = usize::from(walker.is_exclusive());
        let upper = archetypes.len().saturating_sub(skip_count);
        (0, Some(upper))
    }
}

#[derive(Debug, Clone)]
pub struct CompatibleArchetypes<'a> {
    archetypes_after: Option<ArchetypesAfter<'a>>,
}

impl<'a> CompatibleArchetypes<'a> {
    #[inline]
    fn new(
        archetypes: &'a Archetypes,
        graph: &'a Graph,
        archetype: &ErasedArchetype<impl Sized>,
    ) -> Self {
        let archetypes_after = find_archetype(archetypes, archetype)
            .and_then(|start| ArchetypesAfter::new(archetypes, graph, start, false));
        Self { archetypes_after }
    }
}

impl<'a> Iterator for CompatibleArchetypes<'a> {
    type Item = &'a ArchetypeInfo;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { archetypes_after } = self;

        let archetypes_after = archetypes_after.as_mut()?;
        archetypes_after.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { archetypes_after } = self;

        let Some(archetypes_after) = archetypes_after.as_ref() else {
            return (0, Some(0));
        };
        archetypes_after.size_hint()
    }
}

impl FusedIterator for CompatibleArchetypes<'_> {}

#[derive(Debug)]
pub struct CompatibleArchetypesMut<'a> {
    archetypes_after: Option<ArchetypesAfterMut<'a>>,
}

impl<'a> CompatibleArchetypesMut<'a> {
    #[inline]
    fn new(
        archetypes: &'a mut Archetypes,
        graph: &'a Graph,
        archetype: &ErasedArchetype<impl Sized>,
    ) -> Self {
        let archetypes_after = find_archetype(archetypes, archetype)
            .and_then(|start| ArchetypesAfterMut::new(archetypes, graph, start, false));
        Self { archetypes_after }
    }
}

impl<'a> Iterator for CompatibleArchetypesMut<'a> {
    type Item = &'a mut ArchetypeInfo;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { archetypes_after } = self;

        let archetypes_after = archetypes_after.as_mut()?;
        archetypes_after.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { archetypes_after } = self;

        let Some(archetypes_after) = archetypes_after.as_ref() else {
            return (0, Some(0));
        };
        archetypes_after.size_hint()
    }
}

impl FusedIterator for CompatibleArchetypesMut<'_> {}

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
        graph: &'a Graph,
        components: &'ctx ComponentRegistry,
    ) -> Result<Self, ArchetypeError> {
        let archetype = ErasedArchetype::<()>::of::<B>(components)?;
        let me = Self {
            archetypes: CompatibleArchetypes::new(archetypes, graph, &archetype),
            components,
            phantom: PhantomData,
        };
        Ok(me)
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
            ..
        } = self;

        f.debug_struct("BundlesIntoIter")
            .field("archetypes", archetypes)
            .field("inner_front", inner_front)
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
        } = self;

        Self {
            archetypes: archetypes.clone(),
            components,
            inner_front: inner_front.clone(),
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
        } = self;

        loop {
            if let item @ Some(_) = and_then_or_clear(inner_front, Iterator::next) {
                return item;
            }
            match archetypes.next() {
                None => return None,
                Some(info) => *inner_front = Self::new_inner(info, components).into(),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self {
            archetypes,
            inner_front,
            ..
        } = self;

        let (flo, fhi) = inner_front
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let lo = flo;

        match (archetypes.size_hint(), fhi) {
            ((0, Some(0)), Some(a)) => (lo, Some(a)),
            _ => (lo, None),
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
        graph: &'a Graph,
        components: &'ctx ComponentRegistry,
    ) -> Result<Self, ArchetypeError> {
        let archetype = ErasedArchetype::<()>::of::<B>(components)?;
        let me = Self {
            archetypes: CompatibleArchetypesMut::new(archetypes, graph, &archetype),
            components,
            phantom: PhantomData,
        };
        Ok(me)
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
            ..
        } = self;

        f.debug_struct("BundlesIntoIter")
            .field("archetypes", archetypes)
            .field("inner_front", inner_front)
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
        } = self;

        loop {
            if let item @ Some(_) = and_then_or_clear(inner_front, Iterator::next) {
                return item;
            }
            match archetypes.next() {
                None => return None,
                Some(info) => *inner_front = Self::new_inner(info, components).into(),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self {
            archetypes,
            inner_front,
            ..
        } = self;

        let (flo, fhi) = inner_front
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let lo = flo;

        match (archetypes.size_hint(), fhi) {
            ((0, Some(0)), Some(a)) => (lo, Some(a)),
            _ => (lo, None),
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
fn find_archetype(
    archetypes: &Archetypes,
    archetype: &ErasedArchetype<impl Sized>,
) -> Option<ArchetypeId> {
    let key = ArchetypeKey::from_ref(archetype);
    let index = archetypes.get_index_of(key)?;
    Some(archetype_id_from_usize(index))
}

#[inline]
fn get_archetype_info(archetypes: &Archetypes, id: ArchetypeId) -> Option<&ArchetypeInfo> {
    let index = archetype_id_into_usize(id);
    archetypes.get_index(index).map(Deref::deref)
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
    archetypes.get_index_mut2(index).map(DerefMut::deref_mut)
}

#[inline]
#[track_caller]
fn unwrap_archetype_info_mut(archetypes: &mut Archetypes, id: ArchetypeId) -> &mut ArchetypeInfo {
    let Some(info) = get_archetype_info_mut(archetypes, id) else {
        unreachable!("{id} should exist")
    };
    info
}

#[inline]
fn find_location(archetypes: &Archetypes, entity: Entity) -> EntityLocation {
    let index = archetypes
        .iter()
        .position(|info| info.storage().contains(entity));
    let Some(index) = index else {
        return EntityLocation::WithoutComponents;
    };

    let archetype_id = archetype_id_from_usize(index);
    EntityLocation::WithComponents(archetype_id)
}
