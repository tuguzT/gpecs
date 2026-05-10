use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    archetype::{
        erased::{ErasedArchetype, error::ArchetypeError},
        registry::{ArchetypeId, CompatibleArchetypesMut},
        storage::{self, ArchetypeStorage},
    },
    bundle::Bundle,
    component::registry::{
        ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

use super::algo;

pub struct BundlesMut<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    archetypes: CompatibleArchetypesMut<'a>,
    components: ComponentRegistryView<'ctx, M, T>,
    phantom: PhantomData<fn() -> B>,
}

impl<'a, 'ctx, B, M, T> BundlesMut<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    #[inline]
    pub(super) fn new(
        archetypes: &'a mut algo::Archetypes,
        graph: &'a algo::Graph,
        components: ComponentRegistryView<'ctx, M, T>,
    ) -> Result<Self, ArchetypeError> {
        let archetype = ErasedArchetype::<()>::of::<B, M, T>(&components)?;
        let me = Self {
            archetypes: CompatibleArchetypesMut::new(archetypes, graph, archetype.as_view()),
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

impl<B, M, T> Debug for BundlesMut<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetypes, .. } = self;
        f.debug_struct("BundlesMut")
            .field("archetypes", archetypes)
            .finish_non_exhaustive()
    }
}

impl<'a, B, M, T> Iterator for BundlesMut<'a, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    type Item = (ArchetypeId, storage::BundlesMut<'a, B>);

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

impl<B, M, T> FusedIterator for BundlesMut<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
}

fn into_storage_bundles<'a, B>(
    item: (ArchetypeId, &'a mut ArchetypeStorage),
    components: &ComponentRegistryView<
        impl Sized,
        impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
    >,
) -> (ArchetypeId, storage::BundlesMut<'a, B>)
where
    B: Bundle,
{
    let (archetype_id, storage) = item;
    let bundles = storage
        .as_mut_bundles(components)
        .expect("archetype should be compatible with requested bundle");
    (archetype_id, bundles)
}
