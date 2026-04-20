use crate::archetype::registry::ArchetypeId;

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
    pub const fn has_components(self) -> bool {
        matches!(self, Self::WithComponents(..))
    }

    /// Returns `true` if entity has no components attached to it.
    #[inline]
    pub const fn has_no_components(self) -> bool {
        !self.has_components()
    }

    /// Retrieves archetype of some entity if it has components attached to it.
    #[inline]
    pub const fn archetype_id(self) -> Option<ArchetypeId> {
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
