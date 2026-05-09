use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::NonNull,
};

use crate::archetype::{
    registry::{ArchetypeIds, ArchetypeInfo, Iter},
    storage::ArchetypeStorage,
};

use super::algo;

pub struct IterMut<'a> {
    archetypes: &'a mut algo::Archetypes,
    ids: ArchetypeIds,
}

impl<'a> IterMut<'a> {
    #[inline]
    pub(super) fn new(archetypes: &'a mut algo::Archetypes) -> Self {
        let ids = ArchetypeIds::new(archetypes);
        unsafe { Self::from_parts(archetypes, ids) }
    }

    #[inline]
    pub(super) unsafe fn from_parts(
        archetypes: &'a mut algo::Archetypes,
        ids: ArchetypeIds,
    ) -> Self {
        Self { archetypes, ids }
    }
}

impl Debug for IterMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, ids } = self;

        let iter = unsafe { Iter::from_parts(archetypes, ids.clone()) };
        let entries = iter.map(From::from);
        f.debug_map().entries(entries).finish()
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = ArchetypeInfo<&'a mut ArchetypeStorage>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { archetypes, ids } = self;

        let archetype_id = ids.next()?;
        let storage = algo::get_archetype_storage_mut(archetypes, archetype_id)?;

        // SAFETY: `ArchetypeIds` contains unique archetype ids
        let storage = unsafe { NonNull::from_mut(storage).as_mut() };
        let info = ArchetypeInfo::new(archetype_id, storage);
        Some(info)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ids, .. } = self;
        ids.size_hint()
    }
}

impl DoubleEndedIterator for IterMut<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { archetypes, ids } = self;

        let archetype_id = ids.next_back()?;
        let storage = algo::get_archetype_storage_mut(archetypes, archetype_id)?;

        // SAFETY: `ArchetypeIds` contains unique archetype ids
        let storage = unsafe { NonNull::from_mut(storage).as_mut() };
        let info = ArchetypeInfo::new(archetype_id, storage);
        Some(info)
    }
}

impl ExactSizeIterator for IterMut<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { ids, .. } = self;
        ids.len()
    }
}

impl FusedIterator for IterMut<'_> {}
