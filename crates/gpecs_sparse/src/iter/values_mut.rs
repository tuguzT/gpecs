use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    iter::{IterMut, RawValues, RawValuesMut},
    soa::traits::{
        MutPtrs, Ptrs, RawSoa, RefsMut, SliceMutPtrs, SlicePtrs, Slices, SlicesMut, Soa,
    },
};

#[repr(transparent)]
pub struct ValuesMut<'ctx, 'a, K, V>
where
    K: 'ctx,
    V: RawSoa + ?Sized + 'ctx + 'a,
{
    inner: IterMut<'ctx, 'a, K, V>,
}

impl<'ctx, 'a, K, V> ValuesMut<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: IterMut<'ctx, 'a, K, V>) -> Self {
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
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'ctx, V> {
        let (_, value) = self.as_mut_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&'ctx V::Context, MutPtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.as_mut_ptrs_with_context();
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
    pub fn into_mut_ptrs(self) -> MutPtrs<'ctx, V> {
        let (_, value) = self.into_mut_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'ctx V::Context, MutPtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.into_mut_ptrs_with_context();
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
    pub fn as_mut_slice_ptrs(&mut self) -> SliceMutPtrs<'ctx, V> {
        let (_, values) = self.as_mut_slice_ptrs_with_context();
        values
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(&mut self) -> (&'ctx V::Context, SliceMutPtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.as_mut_slice_ptrs_with_context();
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

        let (context, _, values) = inner.into_slice_ptrs_with_context();
        (context, values)
    }

    #[inline]
    pub fn into_mut_slice_ptrs(self) -> SliceMutPtrs<'ctx, V> {
        let (_, values) = self.into_mut_slice_ptrs_with_context();
        values
    }

    #[inline]
    pub fn into_mut_slice_ptrs_with_context(self) -> (&'ctx V::Context, SliceMutPtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, _, values) = inner.into_mut_slice_ptrs_with_context();
        (context, values)
    }

    #[inline]
    pub fn into_raw_values(self) -> RawValues<'ctx, K, V> {
        let Self { inner } = self;
        let inner = inner.into_inner().into_raw_iter();
        RawValues::from_inner(inner)
    }

    #[inline]
    pub fn into_raw_values_mut(self) -> RawValuesMut<'ctx, K, V> {
        let Self { inner } = self;
        let inner = inner.into_inner().into_raw_iter_mut();
        RawValuesMut::from_inner(inner)
    }
}

impl<'ctx, 'a, K, V> ValuesMut<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn into_slices(self) -> SlicesMut<'ctx, 'a, V> {
        let (_, values) = self.into_slices_with_context();
        values
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'ctx V::Context, SlicesMut<'ctx, 'a, V>) {
        let Self { inner } = self;

        let (context, _, value) = inner.into_slices_with_context();
        (context, value)
    }
}

impl<'a, K, V> ValuesMut<'_, '_, K, V>
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

        let (context, _, value) = inner.as_slices_with_context();
        (context, value)
    }
}

impl<K, V> Debug for ValuesMut<'_, '_, K, V>
where
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slices();
        f.debug_tuple("ValuesMut").field(values).finish()
    }
}

impl<T, K, V> AsRef<[T]> for ValuesMut<'_, '_, K, V>
where
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices().into()
    }
}

impl<'ctx, 'a, K, V> Iterator for ValuesMut<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    type Item = RefsMut<'ctx, 'a, V>;

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

impl<'a, K, V> DoubleEndedIterator for ValuesMut<'_, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(_, value)| value)
    }
}

impl<'a, K, V> ExactSizeIterator for ValuesMut<'_, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        ValuesMut::len(self)
    }
}

impl<'a, K, V> FusedIterator for ValuesMut<'_, 'a, K, V> where V: Soa<'a> + ?Sized {}
