use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    archetype::{
        erased::{ErasedArchetype, error::ArchetypeError},
        registry::{ArchetypeInfo, CompatibleArchetypesMut},
        storage::{ArchetypeStorage, BundleIterMut},
    },
    bundle::{Bundle, BundleRefsMut},
    component::registry::{
        ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
    entity::Entity,
    soa::traits::Slices,
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

impl<'a, 'ctx, B, M, T> IntoIterator for BundlesMut<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    type Item = (Entity, BundleRefsMut<'a, B>);

    // FIXME: this actually could be just `FlatMap` with closure,
    // but it cannot be returned because `impl Trait` is unstable in associated types
    type IntoIter = BundlesMutIntoIter<'a, 'ctx, B, M, T>;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            archetypes,
            components,
            ..
        } = self;

        BundlesMutIntoIter {
            archetypes,
            components,
            inner_front: None,
        }
    }
}

#[cfg(feature = "rayon")]
impl<'a, B, M, T> rayon::iter::IntoParallelIterator for BundlesMut<'a, '_, B, M, T>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Send,
    BundleRefsMut<'a, B>: Send,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    type Item = (Entity, BundleRefsMut<'a, B>);
    type Iter = rayon::iter::Flatten<
        rayon::vec::IntoIter<crate::archetype::storage::BundleParIterMut<'a, B>>,
    >;

    fn into_par_iter(self) -> Self::Iter {
        use itertools::Itertools;
        use rayon::prelude::*;

        let Self {
            archetypes,
            ref components,
            ..
        } = self;

        archetypes
            .map(|info| {
                info.into_meta()
                    .bundle_par_iter_mut::<B>(components)
                    .expect("archetype should be compatible with requested bundle")
            })
            .collect_vec()
            .into_par_iter()
            .flatten()
    }
}

pub struct BundlesMutIntoIter<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    archetypes: CompatibleArchetypesMut<'a>,
    components: ComponentRegistryView<'ctx, M, T>,
    inner_front: Option<BundleIterMut<'a, B>>,
}

impl<'a, 'ctx, B, M, T> BundlesMutIntoIter<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    #[inline]
    fn new_inner(
        info: ArchetypeInfo<&'a mut ArchetypeStorage>,
        components: &ComponentRegistryView<'ctx, M, T>,
    ) -> BundleIterMut<'a, B> {
        info.into_meta()
            .bundle_iter_mut(components)
            .expect("archetype should be compatible with requested bundle")
    }
}

impl<B, M, T> Debug for BundlesMutIntoIter<'_, '_, B, M, T>
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

impl<'a, B, M, T> Iterator for BundlesMutIntoIter<'a, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    type Item = (Entity, BundleRefsMut<'a, B>);

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
                Some(info) => *inner_front = Self::new_inner(info, components).into(),
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

impl<B, M, T> FusedIterator for BundlesMutIntoIter<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
}
