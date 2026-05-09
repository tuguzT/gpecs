use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::archetype::{
    registry::{ArchetypeIds, ArchetypeInfo},
    storage::ArchetypeStorage,
};

use super::algo;

#[derive(Clone)]
pub struct Iter<'a> {
    archetypes: &'a algo::Archetypes,
    ids: ArchetypeIds,
}

impl<'a> Iter<'a> {
    #[inline]
    pub(super) fn new(archetypes: &'a algo::Archetypes) -> Self {
        let ids = ArchetypeIds::new(archetypes);
        unsafe { Self::from_parts(archetypes, ids) }
    }

    #[inline]
    pub(super) unsafe fn from_parts(archetypes: &'a algo::Archetypes, ids: ArchetypeIds) -> Self {
        Self { archetypes, ids }
    }
}

impl Debug for Iter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone().map(From::from);
        f.debug_map().entries(entries).finish()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = ArchetypeInfo<&'a ArchetypeStorage>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { archetypes, ids } = self;

        let archetype_id = ids.next()?;
        let storage = algo::get_archetype_storage(archetypes, archetype_id)?;

        let info = ArchetypeInfo::new(archetype_id, storage);
        Some(info)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ids, .. } = self;
        ids.size_hint()
    }
}

impl DoubleEndedIterator for Iter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { archetypes, ids } = self;

        let archetype_id = ids.next_back()?;
        let storage = algo::get_archetype_storage(archetypes, archetype_id)?;

        let info = ArchetypeInfo::new(archetype_id, storage);
        Some(info)
    }
}

impl ExactSizeIterator for Iter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { ids, .. } = self;
        ids.len()
    }
}

impl FusedIterator for Iter<'_> {}
