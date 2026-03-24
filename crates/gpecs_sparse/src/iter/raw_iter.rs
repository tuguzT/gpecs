use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    item::{DenseItem, DenseSlicePtrs},
    iter::{Iter, RawIterMut, RawKeys, RawValues},
    soa::{
        self,
        identity::Identity,
        traits::{Ptrs, RawSoa, SlicePtrs},
    },
};

type Inner<'ctx, K, V> = soa::slice::RawIter<'ctx, DenseItem<K, V>>;

#[repr(transparent)]
pub struct RawIter<'ctx, K, V>
where
    V: RawSoa + ?Sized,
    V::Context: 'ctx,
{
    inner: Inner<'ctx, K, V>,
}

impl<'ctx, K, V> RawIter<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn new(context: &'ctx V::Context, keys: *const [K], values: SlicePtrs<'ctx, V>) -> Self {
        let slices = DenseSlicePtrs::new(context, keys, values);
        let context = Identity::from_inner_ref(context);
        let inner = Inner::new(context, slices);
        Self::from_inner(inner)
    }

    #[inline]
    pub(crate) fn from_inner(inner: Inner<'ctx, K, V>) -> Self {
        Self { inner }
    }

    #[inline]
    pub(super) fn into_inner(self) -> Inner<'ctx, K, V> {
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

        let (context, slices) = inner.into_ptrs_with_context();
        let (key, value) = slices.into_parts();
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
        let (keys, values) = slices.into_parts();
        (context, keys, values)
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
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_raw_keys(self) -> RawKeys<'ctx, K, V> {
        let inner = self.into_inner();
        RawKeys::from_inner(inner)
    }

    #[inline]
    pub fn into_raw_values(self) -> RawValues<'ctx, K, V> {
        let inner = self.into_inner();
        RawValues::from_inner(inner)
    }

    #[inline]
    pub fn cast_mut(self) -> RawIterMut<'ctx, K, V> {
        let Self { inner } = self;
        let inner = inner.cast_mut();
        RawIterMut::from_inner(inner)
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> Iter<'ctx, 'a, K, V> {
        let inner = unsafe { self.into_inner().deref() };
        Iter::from_inner(inner)
    }
}

impl<K, V> Debug for RawIter<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SlicePtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slice_ptrs();
        f.debug_struct("RawIter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Clone for RawIter<'_, K, V>
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

impl<'ctx, K, V> Iterator for RawIter<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = (*const K, Ptrs<'ctx, V>);

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

impl<K, V> DoubleEndedIterator for RawIter<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(From::from)
    }
}

impl<K, V> ExactSizeIterator for RawIter<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        RawIter::len(self)
    }
}

impl<K, V> FusedIterator for RawIter<'_, K, V> where V: RawSoa + ?Sized {}
