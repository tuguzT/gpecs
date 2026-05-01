use core::{
    fmt::{self, Debug},
    slice,
};

use rayon::iter::{
    IndexedParallelIterator, ParallelIterator,
    plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
};

use crate::soa::traits::RawSoa;

pub struct ParKeys<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    context: &'ctx V::Context,
    keys: &'a [K],
}

impl<'ctx, 'a, K, V> ParKeys<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(crate) fn new(context: &'ctx V::Context, keys: &'a [K]) -> Self {
        Self { context, keys }
    }

    #[inline]
    pub fn context(&self) -> &'ctx V::Context {
        let Self { context, .. } = *self;
        context
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [K] {
        let Self { keys, .. } = *self;
        keys
    }

    #[inline]
    pub fn into_parts(self) -> (&'ctx V::Context, &'a [K]) {
        let Self { context, keys } = self;
        (context, keys)
    }
}

impl<K, V> Debug for ParKeys<'_, '_, K, V>
where
    K: Debug,
    V: RawSoa + ?Sized,
    V::Context: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { context, keys } = self;
        f.debug_struct("ParKeys")
            .field("context", context)
            .field("keys", keys)
            .finish()
    }
}

impl<K, V> Clone for ParKeys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { context, keys } = *self;
        Self { context, keys }
    }
}

impl<'a, K, V> ParallelIterator for ParKeys<'_, 'a, K, V>
where
    K: Sync,
    V: RawSoa + ?Sized,
    V::Context: Sync,
{
    type Item = &'a K;

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

impl<K, V> IndexedParallelIterator for ParKeys<'_, '_, K, V>
where
    K: Sync,
    V: RawSoa + ?Sized,
    V::Context: Sync,
{
    fn len(&self) -> usize {
        let Self { keys, .. } = self;
        keys.len()
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

impl<'a, K, V> Producer for ParKeys<'_, 'a, K, V>
where
    K: Sync,
    V: RawSoa + ?Sized,
    V::Context: Sync,
{
    type Item = &'a K;
    type IntoIter = slice::Iter<'a, K>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { keys, .. } = self;
        keys.iter()
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let Self { context, keys } = self;

        let (left, right) = keys.split_at(index);
        let left = Self {
            context,
            keys: left,
        };
        let right = Self {
            context,
            keys: right,
        };
        (left, right)
    }
}
