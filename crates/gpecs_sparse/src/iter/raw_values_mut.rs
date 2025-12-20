use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    item::DenseItem,
    iter::{IterMut, RawIterMut, RawValues, Values, ValuesMut},
    soa::{
        self,
        traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs},
    },
};

#[repr(transparent)]
pub struct RawValuesMut<'c, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c,
{
    inner: RawIterMut<'c, K, V>,
}

impl<'c, K, V> RawValuesMut<'c, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(crate) fn from_inner(inner: soa::slice::RawIterMut<'c, DenseItem<K, V>>) -> Self {
        let inner = RawIterMut::from_inner(inner);
        Self { inner }
    }

    #[inline]
    fn into_inner(self) -> soa::slice::RawIterMut<'c, DenseItem<K, V>> {
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
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'c, V> {
        let (_, value) = self.as_mut_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&'c V::Context, MutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.as_mut_ptrs_with_context();
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
    pub fn into_mut_ptrs(self) -> MutPtrs<'c, V> {
        let (_, value) = self.into_mut_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'c V::Context, MutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.into_mut_ptrs_with_context();
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
    pub fn as_slice_mut_ptrs(&mut self) -> SliceMutPtrs<'c, V> {
        let (_, values) = self.as_slice_mut_ptrs_with_context();
        values
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(&mut self) -> (&'c V::Context, SliceMutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, _, values) = inner.as_slice_mut_ptrs_with_context();
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
    pub fn into_slice_mut_ptrs(self) -> SliceMutPtrs<'c, V> {
        let (_, values) = self.into_slice_mut_ptrs_with_context();
        values
    }

    #[inline]
    pub fn into_slice_mut_ptrs_with_context(self) -> (&'c V::Context, SliceMutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, _, values) = inner.into_slice_mut_ptrs_with_context();
        (context, values)
    }

    #[inline]
    pub fn cast_const(self) -> RawValues<'c, K, V> {
        let inner = self.into_inner().cast_const();
        RawValues::from_inner(inner)
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> Values<'c, 'a, K, V> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ValuesMut<'c, 'a, K, V> {
        let inner = unsafe { self.into_inner().deref_mut() };
        let inner = IterMut::from_inner(inner);
        unsafe { ValuesMut::from_inner(inner) }
    }
}

impl<K, V> Debug for RawValuesMut<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = &self.as_slice_ptrs();
        f.debug_tuple("RawValuesMut").field(slices).finish()
    }
}

impl<K, V> Clone for RawValuesMut<'_, K, V>
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

impl<'c, K, V> Iterator for RawValuesMut<'c, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = MutPtrs<'c, V>;

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

impl<K, V> DoubleEndedIterator for RawValuesMut<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(_, value)| value)
    }
}

impl<K, V> ExactSizeIterator for RawValuesMut<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        RawValuesMut::len(self)
    }
}

impl<K, V> FusedIterator for RawValuesMut<'_, K, V> where V: RawSoa + ?Sized {}
