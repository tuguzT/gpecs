use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    item::DenseItem,
    iter::{RawIter, RawIterMut},
    soa::{
        self,
        traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs, Soa},
    },
};

#[repr(transparent)]
pub struct IterMut<'c, 'a, K, V>
where
    K: 'c + 'a,
    V: RawSoa + ?Sized + 'c + 'a,
{
    inner: soa::slice::IterMut<'c, 'a, DenseItem<K, V>>,
}

impl<'c, 'a, K, V> IterMut<'c, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) fn from_inner(inner: soa::slice::IterMut<'c, 'a, DenseItem<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub(super) fn into_inner(self) -> soa::slice::IterMut<'c, 'a, DenseItem<K, V>> {
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
    pub fn as_mut_ptrs(&mut self) -> (*const K, MutPtrs<'c, V>) {
        let (_, key, value) = self.as_mut_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&'c V::Context, *const K, MutPtrs<'c, V>) {
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

        let (context, ptrs) = inner.into_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> (*const K, MutPtrs<'c, V>) {
        let (_, key, value) = self.into_mut_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'c V::Context, *const K, MutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.into_mut_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
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
    pub fn into_raw_iter(self) -> RawIter<'c, K, V> {
        let Self { inner } = self;
        RawIter::from_inner(inner.into_raw_iter())
    }

    #[inline]
    pub fn into_raw_iter_mut(self) -> RawIterMut<'c, K, V> {
        let Self { inner } = self;
        RawIterMut::from_inner(inner.into_raw_iter_mut())
    }
}

impl<'c, 'a, K, V> IterMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn into_slices(self) -> (&'a [K], V::SlicesMut<'c, 'a>) {
        let (_, keys, values) = self.into_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'c V::Context, &'a [K], V::SlicesMut<'c, 'a>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[K], V::Slices<'_, '_>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&V::Context, &[K], V::Slices<'_, '_>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.as_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }
}

impl<K, V> Debug for IterMut<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slices();
        f.debug_struct("IterMut")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<T, K, V> AsRef<[T]> for IterMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Into<&'any [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        let (_, values) = self.as_slices();
        values.into()
    }
}

impl<'c, 'a, K, V> Iterator for IterMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::RefsMut<'c, 'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|refs| {
            let (key, value) = refs.into_parts();
            (&*key, value)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|refs| {
            let (key, value) = refs.into_parts();
            (&*key, value)
        })
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<K, V> FusedIterator for IterMut<'_, '_, K, V> where V: Soa {}
