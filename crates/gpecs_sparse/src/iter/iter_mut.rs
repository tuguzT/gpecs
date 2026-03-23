use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    item::{DenseContext, DenseItem, DenseSliceMutPtrs, DenseSlicesMut},
    iter::{Keys, RawIter, RawIterMut, ValuesMut},
    soa::{
        self,
        traits::{
            MutPtrs, Ptrs, RawSoa, RefsMut, SliceMutPtrs, SlicePtrs, Slices, SlicesMut, Soa,
            SoaOwned,
        },
    },
};

type Inner<'ctx, 'a, K, V> = soa::slice::IterMut<'ctx, 'a, DenseItem<K, V>>;

#[repr(transparent)]
pub struct IterMut<'ctx, 'a, K, V>
where
    K: 'ctx + 'a,
    V: RawSoa + ?Sized + 'ctx,
{
    inner: Inner<'ctx, 'a, K, V>,
}

impl<'ctx, 'a, K, V> IterMut<'ctx, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    #[track_caller]
    pub unsafe fn from_parts(
        context: &'ctx V::Context,
        keys: *mut [K],
        values: SliceMutPtrs<'ctx, V>,
    ) -> Self {
        let slices = DenseSliceMutPtrs::new(context, keys, values);
        let context = DenseContext::from_inner_ref(context);
        let inner = unsafe { Inner::from_parts(context, slices) };
        Self::from_inner(inner)
    }

    #[inline]
    pub(super) fn from_inner(inner: Inner<'ctx, 'a, K, V>) -> Self {
        Self { inner }
    }

    #[inline]
    pub(super) fn into_inner(self) -> Inner<'ctx, 'a, K, V> {
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
    pub fn context(&self) -> &'ctx V::Context {
        let Self { inner } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const K, Ptrs<'ctx, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'ctx V::Context, *const K, Ptrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*const K, MutPtrs<'ctx, V>) {
        let (_, key, value) = self.as_mut_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&'ctx V::Context, *const K, MutPtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_mut_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_ptrs(self) -> (*const K, Ptrs<'ctx, V>) {
        let (_, key, value) = self.into_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx V::Context, *const K, Ptrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.into_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> (*const K, MutPtrs<'ctx, V>) {
        let (_, key, value) = self.into_mut_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'ctx V::Context, *const K, MutPtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.into_mut_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [K], SlicePtrs<'ctx, V>) {
        let (_, keys, values) = self.as_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'ctx V::Context, *const [K], SlicePtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_mut_slice_ptrs(&mut self) -> (*const [K], SliceMutPtrs<'ctx, V>) {
        let (_, keys, values) = self.as_mut_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(
        &mut self,
    ) -> (&'ctx V::Context, *const [K], SliceMutPtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_mut_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (*const [K], SlicePtrs<'ctx, V>) {
        let (_, keys, values) = self.into_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, *const [K], SlicePtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_mut_slice_ptrs(self) -> (*const [K], SliceMutPtrs<'ctx, V>) {
        let (_, keys, values) = self.into_mut_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_mut_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, *const [K], SliceMutPtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_mut_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'ctx, K, V> {
        let Self { inner } = self;
        RawIter::from_inner(inner.into_raw_iter())
    }

    #[inline]
    pub fn into_raw_iter_mut(self) -> RawIterMut<'ctx, K, V> {
        let Self { inner } = self;
        RawIterMut::from_inner(inner.into_raw_iter_mut())
    }
}

impl<'ctx, 'a, K, V> IterMut<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn new(
        context: &'ctx V::Context,
        keys: &'a mut [K],
        values: SlicesMut<'ctx, 'a, V>,
    ) -> Self {
        let slices = DenseSlicesMut::new(context, keys, values);
        let context = DenseContext::from_inner_ref(context);
        let inner = Inner::new(context, slices);
        Self::from_inner(inner)
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [K], SlicesMut<'ctx, 'a, V>) {
        let (_, keys, values) = self.into_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'ctx V::Context, &'a [K], SlicesMut<'ctx, 'a, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_keys(self) -> Keys<'ctx, 'a, K, V> {
        let inner = self.into_raw_iter().into_raw_keys();
        unsafe { Keys::from_inner(inner) }
    }

    #[inline]
    pub fn into_values_mut(self) -> ValuesMut<'ctx, 'a, K, V> {
        unsafe { ValuesMut::from_inner(self) }
    }
}

impl<'a, K, V> IterMut<'_, '_, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> (&'a [K], Slices<'a, 'a, V>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, &'a [K], Slices<'a, 'a, V>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.as_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }
}

impl<K, V> Debug for IterMut<'_, '_, K, V>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
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
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        let (_, values) = self.as_slices();
        values.into()
    }
}

impl<'ctx, 'a, K, V> Iterator for IterMut<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    type Item = (&'a K, RefsMut<'ctx, 'a, V>);

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

impl<'a, K, V> DoubleEndedIterator for IterMut<'_, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
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

impl<'a, K, V> ExactSizeIterator for IterMut<'_, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<'a, K, V> FusedIterator for IterMut<'_, 'a, K, V> where V: Soa<'a> {}
