use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    archetype::{
        erased::{ErasedArchetype, error::ArchetypeError},
        registry::{ArchetypeInfo, CompatibleArchetypes},
        storage::{ArchetypeStorage, BundleIter},
    },
    bundle::{Bundle, BundleRefs},
    component::registry::{
        ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
    entity::Entity,
    soa::traits::Slices,
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

impl<'a, 'ctx, B, M, T> IntoIterator for Bundles<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    type Item = (Entity, BundleRefs<'a, B>);

    // FIXME: this actually could be just `FlatMap` with closure,
    // but it cannot be returned because `impl Trait` is unstable in associated types
    type IntoIter = BundlesIntoIter<'a, 'ctx, B, M, T>;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            archetypes,
            components,
            ..
        } = self;

        BundlesIntoIter {
            archetypes,
            components,
            inner_front: None,
        }
    }
}

#[cfg(feature = "rayon")]
impl<'a, B, M, T> rayon::iter::IntoParallelIterator for Bundles<'a, '_, B, M, T>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Sync,
    BundleRefs<'a, B>: Send,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    type Item = (Entity, BundleRefs<'a, B>);
    type Iter =
        rayon::iter::Flatten<rayon::vec::IntoIter<crate::archetype::storage::BundleParIter<'a, B>>>;

    fn into_par_iter(self) -> Self::Iter {
        use itertools::Itertools;
        use rayon::prelude::*;

        let Self {
            archetypes,
            ref components,
            ..
        } = self;

        archetypes
            .map(|info| make_par_inner(info, components))
            .collect_vec()
            .into_par_iter()
            .flatten()
    }
}

pub struct BundlesIntoIter<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    archetypes: CompatibleArchetypes<'a>,
    components: ComponentRegistryView<'ctx, M, T>,
    inner_front: Option<BundleIter<'a, B>>,
}

impl<B, M, T> Debug for BundlesIntoIter<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            archetypes,
            inner_front,
            ..
        } = self;

        f.debug_struct("BundlesIntoIter")
            .field("archetypes", archetypes)
            .field("inner_front", inner_front)
            .finish_non_exhaustive()
    }
}

impl<B, M, T> Clone for BundlesIntoIter<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType> + Clone,
{
    fn clone(&self) -> Self {
        let Self {
            archetypes,
            components,
            inner_front,
        } = self;

        Self {
            archetypes: archetypes.clone(),
            components: components.clone(),
            inner_front: inner_front.clone(),
        }
    }
}

impl<'a, B, M, T> Iterator for BundlesIntoIter<'a, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    type Item = (Entity, BundleRefs<'a, B>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            archetypes,
            components,
            inner_front,
        } = self;

        loop {
            if let item @ Some(_) = algo::and_then_or_clear(inner_front, Iterator::next) {
                return item;
            }
            match archetypes.next() {
                None => return None,
                Some(info) => *inner_front = make_inner(info, components).into(),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self {
            archetypes,
            inner_front,
            ..
        } = self;

        let (flo, fhi) = inner_front
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let lo = flo;

        match (archetypes.size_hint(), fhi) {
            ((0, Some(0)), Some(a)) => (lo, Some(a)),
            _ => (lo, None),
        }
    }
}

impl<B, M, T> FusedIterator for BundlesIntoIter<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
}

#[inline]
fn make_inner<'a, B>(
    info: ArchetypeInfo<&'a ArchetypeStorage>,
    components: &ComponentRegistryView<
        impl Sized,
        impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
    >,
) -> BundleIter<'a, B>
where
    B: Bundle,
{
    info.into_meta()
        .bundle_iter(components)
        .expect("archetype should be compatible with requested bundle")
}

#[inline]
#[cfg(feature = "rayon")]
fn make_par_inner<'a, B>(
    info: ArchetypeInfo<&'a ArchetypeStorage>,
    components: &ComponentRegistryView<
        impl Sized,
        impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
    >,
) -> crate::archetype::storage::BundleParIter<'a, B>
where
    B: Bundle,
{
    info.into_meta()
        .bundle_par_iter(components)
        .expect("archetype should be compatible with requested bundle")
}
