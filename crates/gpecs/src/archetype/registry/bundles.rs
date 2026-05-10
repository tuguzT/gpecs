use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    archetype::{
        erased::{ErasedArchetype, error::ArchetypeError},
        registry::{ArchetypeId, CompatibleArchetypes},
        storage::{self, ArchetypeStorage},
    },
    bundle::Bundle,
    component::registry::{
        ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

use super::algo;

pub struct Bundles<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    archetypes: CompatibleArchetypes<'a>,
    components: ComponentRegistryView<'ctx, M, T>,
    phantom: PhantomData<fn() -> B>,
}

impl<'a, 'ctx, B, M, T> Bundles<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    #[inline]
    pub(super) fn new(
        archetypes: &'a algo::Archetypes,
        graph: &'a algo::Graph,
        components: ComponentRegistryView<'ctx, M, T>,
    ) -> Result<Self, ArchetypeError> {
        let archetype = ErasedArchetype::<()>::of::<B, M, T>(&components)?;
        let me = Self {
            archetypes: CompatibleArchetypes::new(archetypes, graph, archetype.as_view()),
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

impl<B, M, T> Debug for Bundles<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, .. } = self;
        f.debug_struct("Bundles")
            .field("archetypes", archetypes)
            .finish_non_exhaustive()
    }
}

impl<B, M, T> Clone for Bundles<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType> + Clone,
{
    fn clone(&self) -> Self {
        let Self {
            ref archetypes,
            ref components,
            phantom,
        } = *self;

        Self {
            archetypes: archetypes.clone(),
            components: components.clone(),
            phantom,
        }
    }
}

impl<'a, B, M, T> Iterator for Bundles<'a, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    type Item = (ArchetypeId, storage::Bundles<'a, B>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut archetypes,
            ref components,
            ..
        } = *self;

        archetypes
            .next()
            .map(|info| into_storage_bundles(info, components))
    }
}

impl<B, M, T> FusedIterator for Bundles<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
}

fn into_storage_bundles<'a, B>(
    item: (ArchetypeId, &'a ArchetypeStorage),
    components: &ComponentRegistryView<
        impl Sized,
        impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
    >,
) -> (ArchetypeId, storage::Bundles<'a, B>)
where
    B: Bundle,
{
    let (archetype_id, storage) = item;
    let bundles = storage
        .as_bundles(components)
        .expect("archetype should be compatible with requested bundle");
    (archetype_id, bundles)
}
