use std::{
    fmt::{self, Debug},
    ptr::NonNull,
};

use crate::archetype::registry::{ArchetypeId, ArchetypeInfo};

use super::algo;

pub struct ArchetypesAfterMut<'a> {
    archetypes: &'a mut algo::Archetypes,
    walker: algo::GraphWalker<&'a algo::Graph>,
}

impl<'a> ArchetypesAfterMut<'a> {
    #[inline]
    pub(super) fn new(
        archetypes: &'a mut algo::Archetypes,
        graph: &'a algo::Graph,
        start: ArchetypeId,
        exclusive: bool,
    ) -> Option<Self> {
        let _ = algo::get_archetype_info(archetypes, start)?;
        let walker = algo::GraphWalker::new(graph, start, exclusive);

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

        algo::graph_dot_scoped(archetypes, walker.graph(), |graph| {
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
        let info = algo::unwrap_archetype_info_mut(archetypes, archetype_id);

        // SAFETY: BFS walker is non-recursive, so it must not yield the same node twice
        let info = unsafe { NonNull::from_mut(info).as_mut() };
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
