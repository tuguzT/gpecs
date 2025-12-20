use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    iter::{Iter, IterMut, RawIter},
    pair::DenseItem,
    soa::{
        self,
        traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs},
    },
};

#[repr(transparent)]
pub struct RawIterMut<'c, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c,
{
    inner: soa::slice::RawIterMut<'c, DenseItem<K, V>>,
}

impl<'c, K, V> RawIterMut<'c, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c,
{
    #[inline]
    pub(crate) fn from_inner(inner: soa::slice::RawIterMut<'c, DenseItem<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub(super) fn into_inner(self) -> soa::slice::RawIterMut<'c, DenseItem<K, V>> {
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
    pub fn context(&self) -> &'c V::Context {
        let Self { inner } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const K, Ptrs<'c, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'c V::Context, *const K, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut K, MutPtrs<'c, V>) {
        let (_, key, value) = self.as_mut_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&'c V::Context, *mut K, MutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_mut_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_ptrs(self) -> (*const K, Ptrs<'c, V>) {
        let (_, key, value) = self.into_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'c V::Context, *const K, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_ptrs_with_context();
        let (key, value) = slices.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> (*mut K, MutPtrs<'c, V>) {
        let (_, key, value) = self.into_mut_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'c V::Context, *mut K, MutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_mut_ptrs_with_context();
        let (key, value) = slices.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [K], SlicePtrs<'c, V>) {
        let (_, keys, values) = self.as_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'c V::Context, *const [K], SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_slice_mut_ptrs(&mut self) -> (*const [K], SliceMutPtrs<'c, V>) {
        let (_, keys, values) = self.as_slice_mut_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(
        &mut self,
    ) -> (&'c V::Context, *const [K], SliceMutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slice_mut_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (*const [K], SlicePtrs<'c, V>) {
        let (_, keys, values) = self.into_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'c V::Context, *const [K], SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_slice_mut_ptrs(self) -> (*const [K], SliceMutPtrs<'c, V>) {
        let (_, keys, values) = self.into_slice_mut_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slice_mut_ptrs_with_context(
        self,
    ) -> (&'c V::Context, *const [K], SliceMutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slice_mut_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn cast_const(self) -> RawIter<'c, K, V> {
        let Self { inner } = self;
        let inner = inner.cast_const();
        RawIter::from_inner(inner)
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> Iter<'c, 'a, K, V> {
        let inner = unsafe { self.into_inner().deref() };
        Iter::from_inner(inner)
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> IterMut<'c, 'a, K, V> {
        let inner = unsafe { self.into_inner().deref_mut() };
        IterMut::from_inner(inner)
    }
}

impl<K, V> Debug for RawIterMut<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slice_ptrs();
        f.debug_struct("RawIterMut")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Clone for RawIterMut<'_, K, V>
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

impl<'c, K, V> Iterator for RawIterMut<'c, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = (*mut K, MutPtrs<'c, V>);

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

impl<K, V> DoubleEndedIterator for RawIterMut<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(From::from)
    }
}

impl<K, V> ExactSizeIterator for RawIterMut<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        RawIterMut::len(self)
    }
}

impl<K, V> FusedIterator for RawIterMut<'_, K, V> where V: RawSoa + ?Sized {}
