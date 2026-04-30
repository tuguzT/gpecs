use core::fmt::{self, Debug};

use rayon::iter::{
    IndexedParallelIterator, ParallelIterator,
    plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
};

use crate::{
    item::DenseItem,
    iter::Iter,
    soa::{
        slice,
        traits::{RawSoa, Refs, Slices, Soa, SoaOwned},
    },
};

type Inner<'ctx, 'a, K, V> = slice::ParIter<'ctx, 'a, DenseItem<K, V>>;

pub struct ParIter<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
    V::Context: 'ctx,
{
    inner: Inner<'ctx, 'a, K, V>,
}

impl<'ctx, 'a, K, V> ParIter<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    pub(crate) fn new(inner: Inner<'ctx, 'a, K, V>) -> Self {
        Self { inner }
    }
}

impl<K, V> Debug for ParIter<'_, '_, K, V>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let (keys, values) = &inner.slices().into_slices().into();
        f.debug_struct("ParIter")
            .field("keys", &keys)
            .field("values", &values)
            .finish()
    }
}

impl<K, V> Clone for ParIter<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<'ctx, 'a, K, V> ParallelIterator for ParIter<'ctx, 'a, K, V>
where
    K: Sync + 'a,
    V: Soa<'a> + ?Sized,
    V::Context: Sync,
    V::Fields: Sync,
    Refs<'ctx, 'a, V>: Send,
{
    type Item = (&'a K, Refs<'ctx, 'a, V>);

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

impl<'ctx, 'a, K, V> IndexedParallelIterator for ParIter<'ctx, 'a, K, V>
where
    K: Sync + 'a,
    V: Soa<'a> + ?Sized,
    V::Context: Sync,
    V::Fields: Sync,
    Refs<'ctx, 'a, V>: Send,
{
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.slices().len()
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

impl<'ctx, 'a, K, V> Producer for ParIter<'ctx, 'a, K, V>
where
    K: Sync + 'a,
    V: Soa<'a> + ?Sized,
    V::Context: Sync,
    V::Fields: Sync,
    Refs<'ctx, 'a, V>: Send,
{
    type Item = (&'a K, Refs<'ctx, 'a, V>);
    type IntoIter = Iter<'ctx, 'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_slices().into_iter();
        Iter::from_inner(inner)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let Self { inner } = self;

        let (left, right) = inner.split_at(index);
        (Self { inner: left }, Self { inner: right })
    }
}
