use std::fmt::{self, Debug};

use petgraph::visit::Reversed;

use crate::archetype::registry::{ArchetypeId, ArchetypeInfo};

use super::algo;

#[derive(Clone)]
pub struct ArchetypesBefore<'a> {
    archetypes: &'a algo::Archetypes,
    walker: algo::GraphWalker<Reversed<&'a algo::Graph>>,
}

impl<'a> ArchetypesBefore<'a> {
    #[inline]
    pub(super) fn new(
        archetypes: &'a algo::Archetypes,
        graph: &'a algo::Graph,
        start: ArchetypeId,
        exclusive: bool,
    ) -> Option<Self> {
        let _ = algo::get_archetype_info(archetypes, start)?;
        let graph = Reversed(graph);
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

impl Debug for ArchetypesBefore<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, walker } = self;

        algo::graph_dot_scoped(archetypes, walker.graph().0, |graph| {
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
        let info = algo::unwrap_archetype_info(archetypes, archetype_id);
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
