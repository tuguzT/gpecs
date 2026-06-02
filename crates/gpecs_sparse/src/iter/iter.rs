#![expect(clippy::module_inception)]

use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_ptr::slice::{CoreSliceItemPtrs, SliceItemPtrs};

use crate::{
    item::{KeyValuePair, KeyValueSlicePtrs, KeyValueSlices},
    iter::{Keys, RawIter, Values},
    soa::{
        self,
        identity::Identity,
        traits::{Ptrs, RawSoa, Refs, SlicePtrs, Slices, Soa, SoaOwned},
    },
};

type Inner<'ctx, 'a, K, V, P> = soa::slice::Iter<'ctx, 'a, KeyValuePair<K, V, P>>;

#[repr(transparent)]
pub struct Iter<'ctx, 'a, K, V, P = CoreSliceItemPtrs<K>>
where
    K: 'a,
    V: RawSoa<Context: 'ctx> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    inner: Inner<'ctx, 'a, K, V, P>,
}

impl<'ctx, 'a, K, V, P> Iter<'ctx, 'a, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    #[track_caller]
    pub unsafe fn from_parts(
        context: &'ctx V::Context,
        keys: *const [K],
        values: SlicePtrs<'ctx, V>,
    ) -> Self {
        let slices = KeyValueSlicePtrs::new(context, keys, values);
        let context = Identity::from_inner_ref(context);
        let inner = unsafe { Inner::from_parts(context, slices) };
        Self::from_inner(inner)
    }

    #[inline]
    pub(super) fn from_inner(inner: Inner<'ctx, 'a, K, V, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub(super) fn into_inner(self) -> Inner<'ctx, 'a, K, V, P> {
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

        let (context, ptrs) = inner.into_ptrs_with_context();
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
        let (key, value) = slices.into_parts();
        (context, key, value)
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
        let (key, value) = slices.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'ctx, K, V, P> {
        let Self { inner } = self;
        RawIter::from_inner(inner.into_raw_iter())
    }
}

impl<'ctx, 'a, K, V, P> Iter<'ctx, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    #[track_caller]
    pub fn new(context: &'ctx V::Context, keys: &'a [K], values: Slices<'ctx, 'a, V>) -> Self {
        let slices = KeyValueSlices::new(context, keys, values);
        let context = Identity::from_inner_ref(context);
        let inner = Inner::new(context, slices);
        Self::from_inner(inner)
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [K], Slices<'ctx, 'a, V>) {
        let (_, keys, values) = self.into_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'ctx V::Context, &'a [K], Slices<'ctx, 'a, V>) {
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
    pub fn into_values(self) -> Values<'ctx, 'a, K, V, P> {
        unsafe { Values::from_inner(self) }
    }
}

impl<'a, K, V, P> Iter<'_, '_, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    pub fn as_slices(&'a self) -> (&'a [K], Slices<'a, 'a, V>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, &'a [K], Slices<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }
}

impl<K, V, P> Debug for Iter<'_, '_, K, V, P>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    P: SliceItemPtrs<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slices();
        f.debug_struct("Iter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, P> Clone for Iter<'_, '_, K, V, P>
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

impl<T, K, V, P> AsRef<[T]> for Iter<'_, '_, K, V, P>
where
    V: SoaOwned + ?Sized,
    P: SliceItemPtrs<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        let (_, values) = self.as_slices();
        values.into()
    }
}

impl<'ctx, 'a, K, V, P> Iterator for Iter<'ctx, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Item = (&'a K, Refs<'ctx, 'a, V>);

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

impl<'a, K, V, P> DoubleEndedIterator for Iter<'_, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(From::from)
    }
}

impl<'a, K, V, P> ExactSizeIterator for Iter<'_, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn len(&self) -> usize {
        Iter::len(self)
    }
}

impl<'a, K, V, P> FusedIterator for Iter<'_, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
}
