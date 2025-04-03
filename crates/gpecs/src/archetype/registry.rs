use std::collections::BTreeSet;

use indexmap::IndexMap;

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

#[derive(Debug, Default)]
pub struct ArchetypeRegistry {
    archetypes: IndexMap<ArchetypeKey, ArchetypeInfo>,
}

impl ArchetypeRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            archetypes: IndexMap::new(),
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
        let Self { archetypes } = self;

        let component_ids = B::component_ids(context, components)?.into_iter().collect();
        if let Some(id) = Self::archetype_id_from_inner(archetypes, &component_ids) {
            return Ok(id);
        }

        let storage = ArchetypeStorage::of::<B>(components, context)
            .expect("component ids of this bundle should be unique");
        let id = Self::register_inner(archetypes, component_ids, storage);
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
        let Self { archetypes } = self;

        let component_ids = try_collect_component_ids(component_ids, ArchetypeKey::insert)?;
        if let Some(id) = Self::archetype_id_from_inner(archetypes, &component_ids) {
            return Ok(id);
        }

        let storage = ArchetypeStorage::new(components, component_ids.iter().copied())
            .expect("component ids of this bundle should be unique");
        let id = Self::register_inner(archetypes, component_ids, storage);
        Ok(id)
    }

    #[inline]
    fn register_inner(
        archetypes: &mut IndexMap<ArchetypeKey, ArchetypeInfo>,
        component_ids: ArchetypeKey,
        storage: ArchetypeStorage,
    ) -> ArchetypeId {
        let id = ArchetypeId(archetypes.len());
        let info = ArchetypeInfo { id, storage };
        if let Some(_) = archetypes.insert(component_ids, info) {
            panic!("duplicate archetype registration")
        }
        id
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { archetypes } = self;
        archetypes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { archetypes } = self;
        archetypes.is_empty()
    }

    #[inline]
    pub fn get_info(&self, id: ArchetypeId) -> Option<&ArchetypeInfo> {
        let Self { archetypes } = self;

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
