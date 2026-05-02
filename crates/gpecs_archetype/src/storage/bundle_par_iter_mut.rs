use core::{
    fmt::{self, Debug},
    mem,
};

use bytemuck::must_cast_slice;
use gpecs_entity::Entity;
use gpecs_sparse::{
    iter::ParIterMut,
    soa::traits::{Slices, SlicesMut},
};
use rayon::iter::{
    IndexedParallelIterator, ParallelIterator,
    plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
};

use crate::{
    bundle::{Bundle, BundleRefsMut, BundleSlices, BundleSlicesMut},
    storage::{BundleIterMut, NoEpochEntity},
};

type Inner<'a, B> = ParIterMut<'static, 'a, NoEpochEntity, B>;

#[repr(transparent)]
pub struct BundleParIterMut<'a, B>
where
    B: Bundle,
{
    inner: Inner<'a, B>,
}

impl<'a, B> BundleParIterMut<'a, B>
where
    B: Bundle,
{
    #[inline]
    pub(super) fn new(inner: Inner<'a, B>) -> Self {
        Self { inner }
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
    pub fn as_mut_slices(&mut self) -> (&[Entity], BundleSlicesMut<'_, B>) {
        let Self { inner } = self;

        let (entities, bundles) = inner.as_mut_slices();
        let entities = must_cast_slice(entities);
        let bundles =
            unsafe { mem::transmute::<SlicesMut<'_, '_, B>, BundleSlicesMut<'_, B>>(bundles) };
        (entities, bundles)
    }
}

impl<B> Debug for BundleParIterMut<'_, B>
where
    B: Bundle,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, bundles) = &self.as_slices();
        f.debug_struct("BundleParIterMut")
            .field("entities", entities)
            .field("bundles", bundles)
            .finish()
    }
}

impl<'a, B> ParallelIterator for BundleParIterMut<'a, B>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Send,
    BundleRefsMut<'a, B>: Send,
{
    type Item = (Entity, BundleRefsMut<'a, B>);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'a, B> IndexedParallelIterator for BundleParIterMut<'a, B>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Send,
    BundleRefsMut<'a, B>: Send,
{
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        callback.callback(self)
    }
}

impl<'a, B> Producer for BundleParIterMut<'a, B>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Send,
    BundleRefsMut<'a, B>: Send,
{
    type Item = (Entity, BundleRefsMut<'a, B>);
    type IntoIter = BundleIterMut<'a, B>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        BundleIterMut::from_inner(inner)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let Self { inner } = self;

        let (left, right) = inner.split_at(index);
        (Self { inner: left }, Self { inner: right })
    }
}
