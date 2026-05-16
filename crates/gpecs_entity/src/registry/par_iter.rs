use core::fmt::{self, Debug};

use gpecs_sparse::{item::SparseItem, soa::identity::Identity};
use rayon::iter::{
    IndexedParallelIterator, ParallelIterator,
    plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
};

use crate::{
    entity::{Entity, EntityEpoch},
    registry::{EntityRegistryView, Iter},
};

#[repr(transparent)]
pub struct ParIter<'a, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    view: EntityRegistryView<'a, Meta, S>,
}

impl<'a, Meta, S> ParIter<'a, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    #[inline]
    pub(super) fn new(view: EntityRegistryView<'a, Meta, S>) -> Self {
        Self { view }
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], &[Meta]) {
        let Self { view } = self;

        let (entities, metas, _) = view.as_slices();
        (entities, metas)
    }

    #[inline]
    pub fn as_entities(&self) -> &[Entity] {
        let (entities, _) = self.as_slices();
        entities
    }

    #[inline]
    pub fn as_metas(&self) -> &[Meta] {
        let (_, metas) = self.as_slices();
        metas
    }
}

impl<Meta, S> Debug for ParIter<'_, Meta, S>
where
    Meta: Debug,
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, metas) = &self.as_slices();
        f.debug_struct("ParIter")
            .field("entities", entities)
            .field("metas", metas)
            .finish()
    }
}

impl<Meta, S> Clone for ParIter<'_, Meta, S>
where
    S: SparseItem<Index = u32, Epoch = EntityEpoch>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { view } = *self;
        Self { view }
    }
}

impl<'a, Meta, S> ParallelIterator for ParIter<'a, Meta, S>
where
    Meta: Sync,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Sync,
{
    type Item = (Entity, &'a Meta);

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

impl<Meta, S> IndexedParallelIterator for ParIter<'_, Meta, S>
where
    Meta: Sync,
    S: SparseItem<Index = u32, Epoch = EntityEpoch> + Sync,
{
    fn len(&self) -> usize {
        let Self { view } = self;
        view.len()
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
        let Self { view } = self;

        let view = unsafe { view.into_inner().as_ref_unchecked() };
        let inner = view.into_par_iter();
        let producer = ParIterProducer { inner };
        callback.callback(producer)
    }
}

#[repr(transparent)]
struct ParIterProducer<'a, Meta> {
    inner: gpecs_sparse::iter::ParIter<'a, 'a, Entity, Identity<Meta>>,
}

impl<'a, Meta> Producer for ParIterProducer<'a, Meta>
where
    Meta: Sync + 'a,
{
    type Item = (Entity, &'a Meta);
    type IntoIter = Iter<'a, Meta>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter().into_raw_iter();
        Iter::from_inner(inner)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let Self { inner } = self;

        let (left, right) = inner.split_at(index);
        (Self { inner: left }, Self { inner: right })
    }
}
