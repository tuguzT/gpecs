use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    item::DenseItem,
    iter::{Keys, RawIter},
    soa::{self, traits::RawSoa},
};

type Inner<'ctx, K, V> = soa::slice::RawIter<'ctx, DenseItem<K, V>>;

#[repr(transparent)]
pub struct RawKeys<'ctx, K, V>
where
    K: 'ctx,
    V: RawSoa + ?Sized + 'ctx,
{
    inner: RawIter<'ctx, K, V>,
}

impl<'ctx, K, V> RawKeys<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(crate) fn from_inner(inner: Inner<'ctx, K, V>) -> Self {
        let inner = RawIter::from_inner(inner);
        Self { inner }
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
    pub fn as_ptr(&self) -> *const K {
        let (_, key) = self.as_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_ptr_with_context(&self) -> (&'ctx V::Context, *const K) {
        let Self { inner } = self;

        let (context, key, _) = inner.as_ptrs_with_context();
        (context, key)
    }

    #[inline]
    pub fn into_ptr(self) -> *const K {
        let (_, key) = self.into_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_ptr_with_context(self) -> (&'ctx V::Context, *const K) {
        let Self { inner } = self;

        let (context, key, _) = inner.into_ptrs_with_context();
        (context, key)
    }

    #[inline]
    pub fn as_slice_ptr(&self) -> *const [K] {
        let (_, keys) = self.as_slice_ptr_with_context();
        keys
    }

    #[inline]
    pub fn as_slice_ptr_with_context(&self) -> (&'ctx V::Context, *const [K]) {
        let Self { inner } = self;

        let (context, keys, _) = inner.as_slice_ptrs_with_context();
        (context, keys)
    }

    #[inline]
    pub fn into_slice_ptr(self) -> *const [K] {
        let (_, keys) = self.into_slice_ptr_with_context();
        keys
    }

    #[inline]
    pub fn into_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [K]) {
        let Self { inner } = self;

        let (context, keys, _) = inner.into_slice_ptrs_with_context();
        (context, keys)
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> Keys<'ctx, 'a, K, V> {
        unsafe { Keys::from_inner(self) }
    }
}

impl<K, V> Debug for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice_ptr();
        f.debug_tuple("RawKeys").field(keys).finish()
    }
}

impl<K, V> Clone for RawKeys<'_, K, V>
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

impl<K, V> Iterator for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = *const K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(key, _)| key)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(key, _)| key)
    }
}

impl<K, V> ExactSizeIterator for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        RawKeys::len(self)
    }
}

impl<K, V> FusedIterator for RawKeys<'_, K, V> where V: RawSoa + ?Sized {}
