use std::iter::FusedIterator;

use crate::archetype::{
    erased::ErasedArchetypeView,
    registry::{ArchetypeInfo, ArchetypesAfterMut},
    storage::ArchetypeStorage,
};

use super::algo;

#[derive(Debug)]
pub struct CompatibleArchetypesMut<'a> {
    archetypes_after: Option<ArchetypesAfterMut<'a>>,
}

impl<'a> CompatibleArchetypesMut<'a> {
    #[inline]
    pub(super) fn new(
        archetypes: &'a mut algo::Archetypes,
        graph: &'a algo::Graph,
        archetype: ErasedArchetypeView<impl Sized>,
    ) -> Self {
        let archetypes_after = algo::find_archetype(archetypes, archetype)
            .and_then(|start| ArchetypesAfterMut::new(archetypes, graph, start, false));
        Self { archetypes_after }
    }
}

impl<'a> Iterator for CompatibleArchetypesMut<'a> {
    type Item = ArchetypeInfo<&'a mut ArchetypeStorage>;

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
