use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    iter::{Iter, RawValues},
    soa::traits::{Ptrs, RawSoa, Refs, SlicePtrs, Slices, Soa, SoaOwned},
};

#[repr(transparent)]
pub struct Values<'ctx, 'a, K, V>
where
    K: 'ctx,
    V: RawSoa + ?Sized + 'ctx,
{
    inner: Iter<'ctx, 'a, K, V>,
}

impl<'ctx, 'a, K, V> Values<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: Iter<'ctx, 'a, K, V>) -> Self {
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
    pub fn as_ptrs(&self) -> Ptrs<'ctx, V> {
        let (_, value) = self.as_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'ctx V::Context, Ptrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.as_ptrs_with_context();
        (context, value)
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'ctx, V> {
        let (_, value) = self.into_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx V::Context, Ptrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.into_ptrs_with_context();
        (context, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'ctx, V> {
        let (_, values) = self.as_slice_ptrs_with_context();
        values
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'ctx V::Context, SlicePtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.as_slice_ptrs_with_context();
        (context, value)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'ctx, V> {
        let (_, values) = self.into_slice_ptrs_with_context();
        values
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'ctx V::Context, SlicePtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.into_slice_ptrs_with_context();
        (context, value)
    }

    #[inline]
    pub fn into_raw_values(self) -> RawValues<'ctx, K, V> {
        let Self { inner } = self;
        let inner = inner.into_inner().into_raw_iter();
        RawValues::from_inner(inner)
    }
}

impl<'ctx, 'a, K, V> Values<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn into_slices(self) -> Slices<'ctx, 'a, V> {
        let (_, values) = self.into_slices_with_context();
        values
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'ctx V::Context, Slices<'ctx, 'a, V>) {
        let Self { inner } = self;
        let (context, _, values) = inner.into_slices_with_context();
        (context, values)
    }
}

impl<'a, K, V> Values<'_, '_, K, V>
where
    V: Soa<'a> + ?Sized,
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
}

impl<K, V> Debug for Values<'_, '_, K, V>
where
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slices();
        f.debug_tuple("Values").field(values).finish()
    }
}

impl<K, V> Clone for Values<'_, '_, K, V>
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

impl<T, K, V> AsRef<[T]> for Values<'_, '_, K, V>
where
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices().into()
    }
}

impl<'ctx, 'a, K, V> Iterator for Values<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    type Item = Refs<'ctx, 'a, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next().map(|(_, value)| value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'_, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next_back().map(|(_, value)| value)
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'_, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Values::len(self)
    }
}

impl<'a, K, V> FusedIterator for Values<'_, 'a, K, V> where V: Soa<'a> + ?Sized {}
