use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use bytemuck::{must_cast_slice, must_cast_slice_mut};
use gpecs_entity::Entity;
use gpecs_sparse::{iter::IterMut, soa::traits::Slices};

use crate::{
    bundle::{
        Bundle, BundleMutPtrs, BundlePtrs, BundleRefsMut, BundleSliceMutPtrs, BundleSlicePtrs,
        BundleSlicesMut,
    },
    storage::NoEpochEntity,
};

type Inner<'a, B> = IterMut<'static, 'a, NoEpochEntity, B>;

#[repr(transparent)]
pub struct BundleIterMut<'a, B>
where
    B: Bundle,
{
    inner: Inner<'a, B>,
}

impl<'a, B> BundleIterMut<'a, B>
where
    B: Bundle,
{
    #[inline]
    pub unsafe fn from_parts(entities: *mut [Entity], bundles: BundleSliceMutPtrs<B>) -> Self {
        let keys = entities as *mut [_];
        let inner = unsafe { Inner::from_parts(B::CONTEXT, keys, bundles) };
        Self { inner }
    }

    #[inline]
    pub fn new(entities: &'a mut [Entity], bundles: BundleSlicesMut<'a, B>) -> Self {
        let keys = must_cast_slice_mut(entities);
        let inner = Inner::new(B::CONTEXT, keys, bundles);
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
    pub fn as_mut_ptrs(&mut self) -> (*const Entity, BundleMutPtrs<B>) {
        let Self { inner } = self;

        let (entity, bundle) = inner.as_mut_ptrs();
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
    pub fn into_mut_ptrs(self) -> (*const Entity, BundleMutPtrs<B>) {
        let Self { inner } = self;

        let (entity, bundle) = inner.into_mut_ptrs();
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
    pub fn as_mut_slice_ptrs(&mut self) -> (*const [Entity], BundleSliceMutPtrs<B>) {
        let Self { inner } = self;

        let (entities, bundles) = inner.as_mut_slice_ptrs();
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
    pub fn into_mut_slice_ptrs(self) -> (*const [Entity], BundleSliceMutPtrs<B>) {
        let Self { inner } = self;

        let (entities, bundles) = inner.into_mut_slice_ptrs();
        let entities = entities as *const [_];
        (entities, bundles)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], Slices<'_, '_, B>) {
        let Self { inner } = self;

        let (entities, bundles) = inner.as_slices();
        let entities = must_cast_slice(entities);
        (entities, bundles)
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [Entity], BundleSlicesMut<'a, B>) {
        let Self { inner } = self;

        let (entities, bundles) = inner.into_slices();
        let entities = must_cast_slice(entities);
        (entities, bundles)
    }
}

impl<B> Debug for BundleIterMut<'_, B>
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

impl<'a, B> Iterator for BundleIterMut<'a, B>
where
    B: Bundle,
{
    type Item = (Entity, BundleRefsMut<'a, B>);

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

impl<B> DoubleEndedIterator for BundleIterMut<'_, B>
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

impl<B> ExactSizeIterator for BundleIterMut<'_, B>
where
    B: Bundle,
{
    #[inline]
    fn len(&self) -> usize {
        BundleIterMut::len(self)
    }
}

impl<B> FusedIterator for BundleIterMut<'_, B> where B: Bundle {}

#[inline]
fn map_inner_item<T>(item: (&NoEpochEntity, T)) -> (Entity, T) {
    let (&entity, value) = item;
    (entity.into(), value)
}
