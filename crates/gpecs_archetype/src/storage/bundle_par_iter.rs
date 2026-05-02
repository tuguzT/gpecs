use core::{
    fmt::{self, Debug},
    mem,
};

use bytemuck::must_cast_slice;
use gpecs_entity::Entity;
use gpecs_sparse::{iter::ParIter, soa::traits::Slices};
use rayon::iter::{
    IndexedParallelIterator, ParallelIterator,
    plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
};

use crate::{
    bundle::{Bundle, BundleRefs, BundleSlices},
    storage::{BundleIter, NoEpochEntity},
};

type Inner<'a, B> = ParIter<'static, 'a, NoEpochEntity, B>;

#[repr(transparent)]
pub struct BundleParIter<'a, B>
where
    B: Bundle,
{
    inner: Inner<'a, B>,
}

impl<'a, B> BundleParIter<'a, B>
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
}

impl<B> Debug for BundleParIter<'_, B>
where
    B: Bundle,
    for<'ctx, 'a> Slices<'ctx, 'a, B>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, bundles) = &self.as_slices();
        f.debug_struct("BundleParIter")
            .field("entities", entities)
            .field("bundles", bundles)
            .finish()
    }
}

impl<B> Clone for BundleParIter<'_, B>
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

impl<'a, B> ParallelIterator for BundleParIter<'a, B>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Sync,
    BundleRefs<'a, B>: Send,
{
    type Item = (Entity, BundleRefs<'a, B>);

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

impl<'a, B> IndexedParallelIterator for BundleParIter<'a, B>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Sync,
    BundleRefs<'a, B>: Send,
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

impl<'a, B> Producer for BundleParIter<'a, B>
where
    B: Bundle,
    B::Context: Sync,
    B::Fields: Sync,
    BundleRefs<'a, B>: Send,
{
    type Item = (Entity, BundleRefs<'a, B>);
    type IntoIter = BundleIter<'a, B>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        BundleIter::from_inner(inner)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let Self { inner } = self;

        let (left, right) = inner.split_at(index);
        (Self { inner: left }, Self { inner: right })
    }
}
