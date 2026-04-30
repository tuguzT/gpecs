use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr,
};

use crate::{iter::Keys, soa::traits::RawSoa};

pub struct RawKeys<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    context: &'ctx V::Context,
    key: *const K,
    len: usize,
}

impl<'ctx, K, V> RawKeys<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(crate) fn new(context: &'ctx V::Context, keys: *const [K]) -> Self {
        Self {
            context,
            key: keys.cast(),
            len: keys.len(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
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
        let (_, key) = self.as_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_ptr_with_context(&self) -> (&'ctx V::Context, *const K) {
        let Self { context, key, .. } = *self;
        (context, key)
    }

    #[inline]
    pub fn into_ptr(self) -> *const K {
        let (_, key) = self.into_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_ptr_with_context(self) -> (&'ctx V::Context, *const K) {
        let Self { context, key, .. } = self;
        (context, key)
    }

    #[inline]
    pub fn as_slice_ptr(&self) -> *const [K] {
        let (_, keys) = self.as_slice_ptr_with_context();
        keys
    }

    #[inline]
    pub fn as_slice_ptr_with_context(&self) -> (&'ctx V::Context, *const [K]) {
        let Self { context, key, len } = *self;

        let keys = ptr::slice_from_raw_parts(key, len);
        (context, keys)
    }

    #[inline]
    pub fn into_slice_ptr(self) -> *const [K] {
        let (_, keys) = self.into_slice_ptr_with_context();
        keys
    }

    #[inline]
    pub fn into_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [K]) {
        let Self { context, key, len } = self;

        let keys = ptr::slice_from_raw_parts(key, len);
        (context, keys)
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> Keys<'ctx, 'a, K, V> {
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
        let Self { context, key, len } = *self;
        Self { context, key, len }
    }
}

impl<K, V> Iterator for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = *const K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { key, len, .. } = self;

        if *len == 0 {
            return None;
        }

        let item = *key;
        *key = unsafe { key.add(1) };
        *len -= 1;
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { len, .. } = *self;
        (len, Some(len))
    }
}

impl<K, V> DoubleEndedIterator for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { key, len, .. } = self;

        if *len == 0 {
            return None;
        }

        let item = unsafe { key.add(*len) };
        *len -= 1;
        Some(item)
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
