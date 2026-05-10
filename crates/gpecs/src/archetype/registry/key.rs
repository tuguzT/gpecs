use std::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use crate::{archetype::erased::ErasedArchetypeView, component::registry::ComponentId};

#[repr(transparent)]
pub struct ArchetypeKey<'a, Meta> {
    archetype: ErasedArchetypeView<'a, Meta>,
}

impl<'a, Meta> ArchetypeKey<'a, Meta> {
    #[inline]
    pub fn new(archetype: ErasedArchetypeView<'a, Meta>) -> Self {
        Self { archetype }
    }

    #[inline]
    pub fn len(self) -> usize {
        let Self { archetype } = self;
        archetype.len()
    }

    #[inline]
    pub fn contains(self, component_id: ComponentId) -> bool {
        let Self { archetype } = self;
        archetype.contains(component_id)
    }

    #[inline]
    pub fn component_ids(self) -> impl Iterator<Item = ComponentId> {
        let Self { archetype } = self;
        archetype
            .into_component_id_ordered_iter()
            .map(|(component_id, _)| component_id)
    }

    #[inline]
    pub fn difference(self, other: ArchetypeKey<impl Sized>) -> impl Iterator<Item = ComponentId> {
        self.component_ids().filter(move |&id| !other.contains(id))
    }
}

impl<Meta> Debug for ArchetypeKey<'_, Meta> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.component_ids();
        f.debug_set().entries(entries).finish()
    }
}

impl<Meta> Clone for ArchetypeKey<'_, Meta> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta> Copy for ArchetypeKey<'_, Meta> {}

impl<Meta, OtherMeta> PartialEq<ArchetypeKey<'_, OtherMeta>> for ArchetypeKey<'_, Meta> {
    fn eq(&self, other: &ArchetypeKey<'_, OtherMeta>) -> bool {
        let other = other.component_ids();
        self.component_ids().eq(other)
    }
}

impl<Meta> Eq for ArchetypeKey<'_, Meta> {}

impl<Meta, OtherMeta> PartialOrd<ArchetypeKey<'_, OtherMeta>> for ArchetypeKey<'_, Meta> {
    fn partial_cmp(&self, other: &ArchetypeKey<'_, OtherMeta>) -> Option<cmp::Ordering> {
        let other = other.component_ids();
        self.component_ids().partial_cmp(other)
    }
}

impl<Meta> Ord for ArchetypeKey<'_, Meta> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let other = other.component_ids();
        self.component_ids().cmp(other)
    }
}

impl<Meta> Hash for ArchetypeKey<'_, Meta> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        self.component_ids()
            .for_each(|component_id| component_id.hash(state));
    }
}
