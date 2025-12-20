use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    iter::{Iter, RawIter, RawValuesMut, Values},
    pair::DenseItem,
    soa::{
        self,
        traits::{Ptrs, RawSoa, SlicePtrs},
    },
};

#[repr(transparent)]
pub struct RawValues<'c, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c,
{
    inner: RawIter<'c, K, V>,
}

impl<'c, K, V> RawValues<'c, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(crate) fn from_inner(inner: soa::slice::RawIter<'c, DenseItem<K, V>>) -> Self {
        let inner = RawIter::from_inner(inner);
        Self { inner }
    }

    #[inline]
    fn into_inner(self) -> soa::slice::RawIter<'c, DenseItem<K, V>> {
        let Self { inner } = self;
        inner.into_inner()
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
    pub fn as_ptrs(&self) -> Ptrs<'c, V> {
        let (_, value) = self.as_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'c V::Context, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.as_ptrs_with_context();
        (context, value)
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'c, V> {
        let (_, value) = self.into_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'c V::Context, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.into_ptrs_with_context();
        (context, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'c, V> {
        let (_, values) = self.as_slice_ptrs_with_context();
        values
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'c V::Context, SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, _, values) = inner.as_slice_ptrs_with_context();
        (context, values)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'c, V> {
        let (_, values) = self.into_slice_ptrs_with_context();
        values
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'c V::Context, SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, _, values) = inner.into_slice_ptrs_with_context();
        (context, values)
    }

    #[inline]
    pub fn cast_mut(self) -> RawValuesMut<'c, K, V> {
        let inner = self.into_inner().cast_mut();
        RawValuesMut::from_inner(inner)
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> Values<'c, 'a, K, V> {
        let inner = unsafe { self.into_inner().deref() };
        let inner = Iter::from_inner(inner);
        unsafe { Values::from_inner(inner) }
    }
}

impl<K, V> Debug for RawValues<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = &self.as_slice_ptrs();
        f.debug_tuple("RawValues").field(slices).finish()
    }
}

impl<K, V> Clone for RawValues<'_, K, V>
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

impl<'c, K, V> Iterator for RawValues<'c, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = Ptrs<'c, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(_, value)| value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for RawValues<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(_, value)| value)
    }
}

impl<K, V> ExactSizeIterator for RawValues<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        RawValues::len(self)
    }
}

impl<K, V> FusedIterator for RawValues<'_, K, V> where V: RawSoa + ?Sized {}
