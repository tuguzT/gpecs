use std::{
    fmt::{self, Debug},
    iter::{self, FusedIterator},
    marker::PhantomData,
    slice,
};

use crate::{
    archetype::{
        erased::{ErasedArchetype, error::ArchetypeError},
        registry::{ArchetypeInfo, CompatibleArchetypes},
        storage::ArchetypeStorage,
    },
    bundle::{Bundle, BundleRefs},
    component::registry::{
        ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
    entity::Entity,
    soa::slice::{Iter as SoaIter, SoaSlices},
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

    // FIXME: this actually should be just `FlatMap`,
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

type BundlesIntoIterInner<'a, B> =
    iter::Zip<iter::Copied<slice::Iter<'a, Entity>>, SoaIter<'static, 'a, B>>;

pub struct BundlesIntoIter<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    archetypes: CompatibleArchetypes<'a>,
    components: ComponentRegistryView<'ctx, M, T>,
    inner_front: Option<BundlesIntoIterInner<'a, B>>,
}

impl<'a, 'ctx, B, M, T> BundlesIntoIter<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    #[inline]
    fn new_inner(
        info: ArchetypeInfo<&'a ArchetypeStorage>,
        components: &ComponentRegistryView<'ctx, M, T>,
    ) -> BundlesIntoIterInner<'a, B> {
        let (entities, bundles, _) = info
            .into_meta()
            .as_bundles_with_archetype::<B, T>(components)
            .expect("archetype should be compatible with requested bundle");

        let entities = entities.iter().copied();
        let bundles = SoaSlices::new(B::CONTEXT, bundles);
        entities.zip(bundles)
    }
}

impl<'a, B, M, T> Debug for BundlesIntoIter<'a, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
    BundlesIntoIterInner<'a, B>: Debug,
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

impl<B, M, T> FusedIterator for BundlesIntoIter<'_, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
}
