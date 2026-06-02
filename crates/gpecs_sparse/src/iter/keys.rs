use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    slice,
};

use crate::{iter::RawKeys, soa::traits::RawSoa};

pub struct Keys<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    context: &'ctx V::Context,
    keys: slice::Iter<'a, K>,
}

impl<'ctx, 'a, K, V> Keys<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) fn new(context: &'ctx V::Context, keys: &'a [K]) -> Self {
        let keys = keys.iter();
        Self { context, keys }
    }

    #[inline]
    pub(super) unsafe fn from_inner(inner: RawKeys<'ctx, K, V>) -> Self {
        let (context, keys) = inner.into_slice_ptr_with_context();
        let keys = unsafe { keys.as_ref_unchecked() };
        Self::new(context, keys)
    }

    #[inline]
    pub fn into_raw_keys(self) -> RawKeys<'ctx, K, V> {
        let Self { context, keys } = self;
        RawKeys::new(context, keys.as_slice())
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { keys, .. } = self;
        keys.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &'ctx V::Context {
        let Self { context, .. } = *self;
        context
    }

    #[inline]
    pub fn as_ptr(&self) -> *const K {
        let (_, keys) = self.as_ptr_with_context();
        keys
    }

    #[inline]
    pub fn as_ptr_with_context(&self) -> (&'ctx V::Context, *const K) {
        let Self { context, keys } = self;
        (context, keys.as_slice().as_ptr())
    }

    #[inline]
    pub fn into_ptr(self) -> *const K {
        let (_, keys) = self.into_ptr_with_context();
        keys
    }

    #[inline]
    pub fn into_ptr_with_context(self) -> (&'ctx V::Context, *const K) {
        let Self { context, keys } = self;
        (context, keys.as_slice().as_ptr())
    }

    #[inline]
    pub fn as_slice_ptr(&self) -> *const [K] {
        let (_, keys) = self.as_slice_ptr_with_context();
        keys
    }

    #[inline]
    pub fn as_slice_ptr_with_context(&self) -> (&'ctx V::Context, *const [K]) {
        let Self { context, keys } = self;
        (context, keys.as_slice())
    }

    #[inline]
    pub fn into_slice_ptr(self) -> *const [K] {
        let (_, keys) = self.into_slice_ptr_with_context();
        keys
    }

    #[inline]
    pub fn into_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [K]) {
        let Self { context, keys } = self;
        (context, keys.as_slice())
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
        let Self { ref keys, context } = *self;

        let keys = keys.clone();
        Self { context, keys }
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
        let Self { keys, .. } = self;
        keys.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, .. } = self;
        keys.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for Keys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.next_back()
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

impl<K, V> FusedIterator for Keys<'_, '_, K, V> where V: RawSoa + ?Sized {}
