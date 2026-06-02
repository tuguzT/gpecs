use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_ptr::slice::{CoreSliceItemPtrs, SliceItemPtrs};

use crate::{
    item::{KeyValuePair, KeyValueSlicePtrs},
    iter::{Iter, RawIterMut, RawKeys, RawValues},
    soa::{
        self,
        identity::Identity,
        traits::{Ptrs, RawSoa, SlicePtrs},
    },
};

type Inner<'ctx, K, V, P> = soa::slice::RawIter<'ctx, KeyValuePair<K, V, P>>;

#[repr(transparent)]
pub struct RawIter<'ctx, K, V, P = CoreSliceItemPtrs<K>>
where
    V: RawSoa<Context: 'ctx> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    inner: Inner<'ctx, K, V, P>,
}

impl<'ctx, K, V, P> RawIter<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    #[track_caller]
    pub fn new(context: &'ctx V::Context, keys: *const [K], values: SlicePtrs<'ctx, V>) -> Self {
        let slices = KeyValueSlicePtrs::new(context, keys, values);
        let context = Identity::from_inner_ref(context);
        let inner = Inner::new(context, slices);
        Self::from_inner(inner)
    }

    #[inline]
    pub(crate) fn from_inner(inner: Inner<'ctx, K, V, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub(super) fn into_inner(self) -> Inner<'ctx, K, V, P> {
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
    pub fn as_ptrs(&self) -> (P::Const, Ptrs<'ctx, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'ctx V::Context, P::Const, Ptrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_ptrs(self) -> (P::Const, Ptrs<'ctx, V>) {
        let (_, key, value) = self.into_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx V::Context, P::Const, Ptrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_ptrs_with_context();
        let (key, value) = slices.into_parts();
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
    pub fn into_raw_keys(self) -> RawKeys<'ctx, K, V> {
        let (context, slices) = self.into_inner().into_slice_ptrs_with_context();
        let (keys, _) = slices.into();
        RawKeys::new(context.as_inner(), keys)
    }

    #[inline]
    pub fn into_raw_values(self) -> RawValues<'ctx, K, V, P> {
        let inner = self.into_inner();
        RawValues::from_inner(inner)
    }

    #[inline]
    pub fn cast_mut(self) -> RawIterMut<'ctx, K, V, P> {
        let Self { inner } = self;
        let inner = inner.cast_mut();
        RawIterMut::from_inner(inner)
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> Iter<'ctx, 'a, K, V, P> {
        let inner = unsafe { self.into_inner().as_ref_unchecked() };
        Iter::from_inner(inner)
    }
}

impl<K, V, P> Debug for RawIter<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    for<'ctx> SlicePtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slice_ptrs();
        f.debug_struct("RawIter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, P> Clone for RawIter<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<'ctx, K, V, P> Iterator for RawIter<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Item = (P::Const, Ptrs<'ctx, V>);

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

impl<K, V, P> DoubleEndedIterator for RawIter<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(From::from)
    }
}

impl<K, V, P> ExactSizeIterator for RawIter<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn len(&self) -> usize {
        RawIter::len(self)
    }
}

impl<K, V, P> FusedIterator for RawIter<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
}
