use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem,
};

use bytemuck::must_cast_slice;
use gpecs_entity::{Entity, NoEpochEntity};
use gpecs_sparse::{iter::Iter, soa::traits::Slices};

use crate::bundle::{Bundle, BundlePtrs, BundleRefs, BundleSlicePtrs, BundleSlices};

type Inner<'a, B> = Iter<'static, 'a, NoEpochEntity, B>;

#[repr(transparent)]
pub struct BundleIter<'a, B>
where
    B: Bundle,
{
    inner: Inner<'a, B>,
}

impl<'a, B> BundleIter<'a, B>
where
    B: Bundle,
{
    #[inline]
    pub unsafe fn from_parts(entities: *const [Entity], bundles: BundleSlicePtrs<B>) -> Self {
        let keys = entities as *const [_];
        let inner = unsafe { Inner::from_parts(B::CONTEXT, keys, bundles) };
        Self::from_inner(inner)
    }

    #[inline]
    pub fn new(entities: &'a [Entity], bundles: BundleSlices<'a, B>) -> Self {
        let keys = must_cast_slice(entities);
        let inner = Inner::new(B::CONTEXT, keys, bundles);
        Self::from_inner(inner)
    }

    #[inline]
    pub(super) fn from_inner(inner: Inner<'a, B>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const Entity, BundlePtrs<B>) {
        let Self { inner } = self;

        let (entity, bundle) = inner.as_ptrs();
        let entity = entity.cast();
        (entity, bundle)
    }

    #[inline]
    pub fn into_ptrs(self) -> (*const Entity, BundlePtrs<B>) {
        let Self { inner } = self;

        let (entity, bundle) = inner.into_ptrs();
        let entity = entity.cast();
        (entity, bundle)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [Entity], BundleSlicePtrs<B>) {
        let Self { inner } = self;

        let (entities, bundles) = inner.as_slice_ptrs();
        let entities = entities as *const [_];
        (entities, bundles)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (*const [Entity], BundleSlicePtrs<B>) {
        let Self { inner } = self;

        let (entities, bundles) = inner.into_slice_ptrs();
        let entities = entities as *const [_];
        (entities, bundles)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], BundleSlices<'_, B>) {
        let Self { inner } = self;

        let (entities, bundles) = inner.as_slices();
        let entities = must_cast_slice(entities);
        let bundles = unsafe { mem::transmute::<Slices<'_, '_, B>, BundleSlices<'_, B>>(bundles) };
        (entities, bundles)
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [Entity], BundleSlices<'a, B>) {
        let Self { inner } = self;

        let (entities, bundles) = inner.into_slices();
        let entities = must_cast_slice(entities);
        (entities, bundles)
    }
}

impl<B> Debug for BundleIter<'_, B>
where
    B: Bundle,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, bundles) = &self.as_slices();
        f.debug_struct("BundleIter")
            .field("entities", entities)
            .field("bundles", bundles)
            .finish()
    }
}

impl<B> Clone for BundleIter<'_, B>
where
    B: Bundle,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<'a, B> Iterator for BundleIter<'a, B>
where
    B: Bundle,
{
    type Item = (Entity, BundleRefs<'a, B>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(map_inner_item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(map_inner_item)
    }
}

impl<B> DoubleEndedIterator for BundleIter<'_, B>
where
    B: Bundle,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(map_inner_item)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(map_inner_item)
    }
}

impl<B> ExactSizeIterator for BundleIter<'_, B>
where
    B: Bundle,
{
    #[inline]
    fn len(&self) -> usize {
        BundleIter::len(self)
    }
}

impl<B> FusedIterator for BundleIter<'_, B> where B: Bundle {}

#[inline]
fn map_inner_item<T>(item: (&NoEpochEntity, T)) -> (Entity, T) {
    let (&entity, value) = item;
    (entity.into(), value)
}
