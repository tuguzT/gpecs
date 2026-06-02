use core::fmt::{self, Debug};

use gpecs_ptr::slice::{CoreSliceItemPtrs, SliceItemPtrs};
use rayon::iter::{
    IndexedParallelIterator, ParallelIterator,
    plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
};

use crate::{
    iter::{ParIterMut, ValuesMut},
    soa::traits::{RawSoa, RefsMut, Slices, SlicesMut, Soa, SoaOwned},
};

#[repr(transparent)]
pub struct ParValuesMut<'ctx, 'a, K, V, P = CoreSliceItemPtrs<K>>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    inner: ParIterMut<'ctx, 'a, K, V, P>,
}

impl<'ctx, 'a, K, V, P> ParValuesMut<'ctx, 'a, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    pub(crate) fn new(inner: ParIterMut<'ctx, 'a, K, V, P>) -> Self {
        Self { inner }
    }
}

impl<'a, K, V, P> ParValuesMut<'_, '_, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    pub fn as_slices(&'a self) -> Slices<'a, 'a, V> {
        let (_, values) = self.as_slices_with_context();
        values
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, Slices<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, _, values) = inner.as_slices_with_context();
        (context, values)
    }

    #[inline]
    pub fn as_mut_slices(&'a mut self) -> SlicesMut<'a, 'a, V> {
        let (_, values) = self.as_mut_slices_with_context();
        values
    }

    #[inline]
    pub fn as_mut_slices_with_context(&'a mut self) -> (&'a V::Context, SlicesMut<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, _, values) = inner.as_mut_slices_with_context();
        (context, values)
    }
}

impl<K, V, P> Debug for ParValuesMut<'_, '_, K, V, P>
where
    V: SoaOwned + ?Sized,
    P: SliceItemPtrs<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slices();
        f.debug_tuple("ParValues").field(values).finish()
    }
}

impl<'ctx, 'a, K, V, P> ParallelIterator for ParValuesMut<'ctx, 'a, K, V, P>
where
    K: Send + Sync + 'a,
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    V::Context: Sync,
    V::Fields: Send,
    RefsMut<'ctx, 'a, V>: Send,
{
    type Item = RefsMut<'ctx, 'a, V>;

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

impl<'ctx, 'a, K, V, P> IndexedParallelIterator for ParValuesMut<'ctx, 'a, K, V, P>
where
    K: Send + Sync + 'a,
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    V::Context: Sync,
    V::Fields: Send,
    RefsMut<'ctx, 'a, V>: Send,
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

impl<'ctx, 'a, K, V, P> Producer for ParValuesMut<'ctx, 'a, K, V, P>
where
    K: Send + Sync + 'a,
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    V::Context: Sync,
    V::Fields: Send,
    RefsMut<'ctx, 'a, V>: Send,
{
    type Item = RefsMut<'ctx, 'a, V>;
    type IntoIter = ValuesMut<'ctx, 'a, K, V, P>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ValuesMut::from_inner(inner) }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let Self { inner } = self;

        let (left, right) = inner.split_at(index);
        (Self { inner: left }, Self { inner: right })
    }
}
