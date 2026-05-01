use core::fmt::{self, Debug};

use rayon::iter::{
    IndexedParallelIterator, ParallelIterator,
    plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
};

use crate::{
    item::DenseItem,
    iter::IterMut,
    soa::{
        slice,
        traits::{RawSoa, RefsMut, Slices, SlicesMut, Soa, SoaOwned},
    },
};

type Inner<'ctx, 'a, K, V> = slice::ParIterMut<'ctx, 'a, DenseItem<K, V>>;

pub struct ParIterMut<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
    V::Context: 'ctx,
{
    inner: Inner<'ctx, 'a, K, V>,
}

impl<'ctx, 'a, K, V> ParIterMut<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    pub(crate) fn new(inner: Inner<'ctx, 'a, K, V>) -> Self {
        Self { inner }
    }
}

impl<'a, K, V> ParIterMut<'_, '_, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> (&'a [K], Slices<'a, 'a, V>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, &'a [K], Slices<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.slices().into_slices_with_context();
        let (keys, values) = slices.into();
        (context, keys, values)
    }

    #[inline]
    pub fn as_mut_slices(&'a mut self) -> (&'a [K], SlicesMut<'a, 'a, V>) {
        let (_, keys, values) = self.as_mut_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_mut_slices_with_context(
        &'a mut self,
    ) -> (&'a V::Context, &'a [K], SlicesMut<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.mut_slices().into_slices_with_context();
        let (keys, values) = slices.into();
        (context, keys, values)
    }
}

impl<K, V> Debug for ParIterMut<'_, '_, K, V>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slices();
        f.debug_struct("ParIter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'ctx, 'a, K, V> ParallelIterator for ParIterMut<'ctx, 'a, K, V>
where
    K: Send + Sync + 'a,
    V: Soa<'a> + ?Sized,
    V::Context: Sync,
    V::Fields: Send,
    RefsMut<'ctx, 'a, V>: Send,
{
    type Item = (&'a K, RefsMut<'ctx, 'a, V>);

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

impl<'ctx, 'a, K, V> IndexedParallelIterator for ParIterMut<'ctx, 'a, K, V>
where
    K: Send + Sync + 'a,
    V: Soa<'a> + ?Sized,
    V::Context: Sync,
    V::Fields: Send,
    RefsMut<'ctx, 'a, V>: Send,
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

impl<'ctx, 'a, K, V> Producer for ParIterMut<'ctx, 'a, K, V>
where
    K: Send + Sync + 'a,
    V: Soa<'a> + ?Sized,
    V::Context: Sync,
    V::Fields: Send,
    RefsMut<'ctx, 'a, V>: Send,
{
    type Item = (&'a K, RefsMut<'ctx, 'a, V>);
    type IntoIter = IterMut<'ctx, 'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_slices().into_iter();
        IterMut::from_inner(inner)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let Self { inner } = self;

        let (left, right) = inner.split_at(index);
        (Self { inner: left }, Self { inner: right })
    }
}
