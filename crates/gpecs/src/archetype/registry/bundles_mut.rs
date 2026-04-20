use std::{
    fmt::{self, Debug},
    iter::{self, FusedIterator},
    marker::PhantomData,
    slice,
};

use crate::{
    archetype::{
        erased::{ErasedArchetype, error::ArchetypeError},
        registry::{ArchetypeInfo, CompatibleArchetypesMut},
    },
    bundle::{Bundle, BundleRefsMut},
    component::registry::{
        ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
    entity::Entity,
    soa::slice::{IterMut as SoaIterMut, SoaSlicesMut},
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

    // this actually should be just `FlatMap`,
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

type BundlesMutIntoIterInner<'a, B> =
    iter::Zip<iter::Copied<slice::Iter<'a, Entity>>, SoaIterMut<'static, 'a, B>>;

pub struct BundlesMutIntoIter<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    archetypes: CompatibleArchetypesMut<'a>,
    components: ComponentRegistryView<'ctx, M, T>,
    inner_front: Option<BundlesMutIntoIterInner<'a, B>>,
}

impl<'a, 'ctx, B, M, T> BundlesMutIntoIter<'a, 'ctx, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
{
    #[inline]
    fn new_inner(
        info: &'a mut ArchetypeInfo,
        components: &ComponentRegistryView<'ctx, M, T>,
    ) -> BundlesMutIntoIterInner<'a, B> {
        let storage = unsafe { info.storage_mut() };
        let (entities, bundles, _) = storage
            .as_mut_bundles_with_archetype::<B, T>(components)
            .expect("archetype should be compatible with requested bundle");

        let entities = entities.iter().copied();
        let bundles = SoaSlicesMut::new(B::CONTEXT, bundles);
        entities.zip(bundles)
    }
}

impl<'a, B, M, T> Debug for BundlesMutIntoIter<'a, '_, B, M, T>
where
    B: Bundle,
    T: ComponentIdFrom<Key: FromComponentType>,
    BundlesMutIntoIterInner<'a, B>: Debug,
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
