#![expect(clippy::module_inception)]

use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    item::DenseItem,
    iter::RawIter,
    soa::{
        self,
        traits::{Ptrs, RawSoa, SlicePtrs, Soa},
    },
};

#[repr(transparent)]
pub struct Iter<'ctx, 'a, K, V>
where
    K: 'ctx + 'a,
    V: RawSoa + ?Sized + 'ctx + 'a,
{
    inner: soa::slice::Iter<'ctx, 'a, DenseItem<K, V>>,
}

impl<'ctx, 'a, K, V> Iter<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) fn from_inner(inner: soa::slice::Iter<'ctx, 'a, DenseItem<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub(super) fn into_inner(self) -> soa::slice::Iter<'ctx, 'a, DenseItem<K, V>> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &'ctx V::Context {
        let Self { inner } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const K, Ptrs<'ctx, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'ctx V::Context, *const K, Ptrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_ptrs(self) -> (*const K, Ptrs<'ctx, V>) {
        let (_, key, value) = self.into_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx V::Context, *const K, Ptrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.into_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [K], SlicePtrs<'ctx, V>) {
        let (_, keys, values) = self.as_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'ctx V::Context, *const [K], SlicePtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let (key, value) = slices.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (*const [K], SlicePtrs<'ctx, V>) {
        let (_, keys, values) = self.into_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, *const [K], SlicePtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slice_ptrs_with_context();
        let (key, value) = slices.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'ctx, K, V> {
        let Self { inner } = self;
        RawIter::from_inner(inner.into_raw_iter())
    }
}

impl<'ctx, 'a, K, V> Iter<'ctx, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn as_slices(&self) -> (&[K], V::Slices<'_, '_>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&'ctx V::Context, &[K], V::Slices<'_, '_>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [K], V::Slices<'ctx, 'a>) {
        let (_, keys, values) = self.into_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'ctx V::Context, &'a [K], V::Slices<'ctx, 'a>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }
}

impl<K, V> Debug for Iter<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Slices<'ctx, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slices();
        f.debug_struct("Iter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Clone for Iter<'_, '_, K, V>
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

impl<T, K, V> AsRef<[T]> for Iter<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Slices<'ctx, 'a>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        let (_, values) = self.as_slices();
        values.into()
    }
}

impl<'ctx, 'a, K, V> Iterator for Iter<'ctx, 'a, K, V>
where
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::Refs<'ctx, 'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(From::from)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(From::from)
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Iter::len(self)
    }
}

impl<K, V> FusedIterator for Iter<'_, '_, K, V> where V: Soa {}
