use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    iter::{IterMut, RawValues, RawValuesMut},
    soa::traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs, Soa},
};

#[repr(transparent)]
pub struct ValuesMut<'c, 'a, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c + 'a,
{
    inner: IterMut<'c, 'a, K, V>,
}

impl<'c, 'a, K, V> ValuesMut<'c, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: IterMut<'c, 'a, K, V>) -> Self {
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

        let (context, _, value) = inner.as_slice_ptrs_with_context();
        (context, value)
    }

    #[inline]
    pub fn as_slice_mut_ptrs(&mut self) -> SliceMutPtrs<'c, V> {
        let (_, values) = self.as_slice_mut_ptrs_with_context();
        values
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(&mut self) -> (&'c V::Context, SliceMutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.as_slice_mut_ptrs_with_context();
        (context, value)
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
    pub fn into_raw_values(self) -> RawValues<'c, K, V> {
        let Self { inner } = self;
        let inner = inner.into_inner().into_raw_iter();
        RawValues::from_inner(inner)
    }

    #[inline]
    pub fn into_raw_values_mut(self) -> RawValuesMut<'c, K, V> {
        let Self { inner } = self;
        let inner = inner.into_inner().into_raw_iter_mut();
        RawValuesMut::from_inner(inner)
    }
}

impl<'c, 'a, K, V> ValuesMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn into_slices(self) -> V::SlicesMut<'c, 'a> {
        let (_, values) = self.into_slices_with_context();
        values
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'c V::Context, V::SlicesMut<'c, 'a>) {
        let Self { inner } = self;

        let (context, _, value) = inner.into_slices_with_context();
        (context, value)
    }

    #[inline]
    pub fn as_slices(&self) -> V::Slices<'_, '_> {
        let (_, values) = self.as_slices_with_context();
        values
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&V::Context, V::Slices<'_, '_>) {
        let Self { inner } = self;

        let (context, _, value) = inner.as_slices_with_context();
        (context, value)
    }
}

impl<K, V> Debug for ValuesMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slices();
        f.debug_tuple("ValuesMut").field(values).finish()
    }
}

impl<T, K, V> AsRef<[T]> for ValuesMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Into<&'any [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices().into()
    }
}

impl<'c, 'a, K, V> Iterator for ValuesMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    type Item = V::RefsMut<'c, 'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(_, value)| value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for ValuesMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(_, value)| value)
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        ValuesMut::len(self)
    }
}

impl<K, V> FusedIterator for ValuesMut<'_, '_, K, V> where V: Soa {}
