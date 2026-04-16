use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{iter::RawKeys, soa::traits::RawSoa};

#[repr(transparent)]
pub struct Keys<'ctx, 'a, K, V>
where
    K: 'a,
    V: RawSoa + ?Sized,
{
    inner: RawKeys<'ctx, K, V>,
    phantom: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'ctx, 'a, K, V> Keys<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: RawKeys<'ctx, K, V>) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_raw_keys(self) -> RawKeys<'ctx, K, V> {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &'ctx V::Context {
        let Self { inner, .. } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const K {
        let (_, keys) = self.as_ptr_with_context();
        keys
    }

    #[inline]
    pub fn as_ptr_with_context(&self) -> (&'ctx V::Context, *const K) {
        let Self { inner, .. } = self;

        let (context, key) = inner.as_ptr_with_context();
        (context, key)
    }

    #[inline]
    pub fn into_ptr(self) -> *const K {
        let (_, keys) = self.into_ptr_with_context();
        keys
    }

    #[inline]
    pub fn into_ptr_with_context(self) -> (&'ctx V::Context, *const K) {
        let Self { inner, .. } = self;

        let (context, key) = inner.into_ptr_with_context();
        (context, key)
    }

    #[inline]
    pub fn as_slice_ptr(&self) -> *const [K] {
        let (_, keys) = self.as_slice_ptr_with_context();
        keys
    }

    #[inline]
    pub fn as_slice_ptr_with_context(&self) -> (&'ctx V::Context, *const [K]) {
        let Self { inner, .. } = self;

        let (context, key) = inner.as_slice_ptr_with_context();
        (context, key)
    }

    #[inline]
    pub fn into_slice_ptr(self) -> *const [K] {
        let (_, keys) = self.into_slice_ptr_with_context();
        keys
    }

    #[inline]
    pub fn into_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [K]) {
        let Self { inner, .. } = self;

        let (context, key) = inner.into_slice_ptr_with_context();
        (context, key)
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [K] {
        let (_, keys) = self.as_slice_with_context();
        keys
    }

    #[inline]
    pub fn as_slice_with_context(&self) -> (&'ctx V::Context, &'a [K]) {
        let (context, keys) = self.as_slice_ptr_with_context();
        let keys = unsafe { keys.as_ref_unchecked() };
        (context, keys)
    }

    #[inline]
    pub fn into_slice(self) -> &'a [K] {
        let (_, keys) = self.into_slice_with_context();
        keys
    }

    #[inline]
    pub fn into_slice_with_context(self) -> (&'ctx V::Context, &'a [K]) {
        let (context, keys) = self.into_slice_ptr_with_context();
        let keys = unsafe { keys.as_ref_unchecked() };
        (context, keys)
    }
}

impl<K, V> Debug for Keys<'_, '_, K, V>
where
    K: Debug,
    V: RawSoa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("Keys").field(keys).finish()
    }
}

impl<K, V> Clone for Keys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;

        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<K, V> AsRef<[K]> for Keys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for Keys<'_, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next().map(|key| unsafe { key.as_ref_unchecked() })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for Keys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner
            .next_back()
            .map(|key| unsafe { key.as_ref_unchecked() })
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Keys::len(self)
    }
}

impl<K, V> FusedIterator for Keys<'_, '_, K, V> where V: RawSoa {}
