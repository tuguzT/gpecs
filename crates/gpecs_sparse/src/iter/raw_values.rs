use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_ptr::slice::{CoreSliceItemPtrs, SliceItemPtrs};

use crate::{
    item::KeyValuePair,
    iter::{Iter, RawIter, RawValuesMut, Values},
    soa::{
        self,
        traits::{Ptrs, RawSoa, SlicePtrs},
    },
};

type Inner<'ctx, K, V, P> = soa::slice::RawIter<'ctx, KeyValuePair<K, V, P>>;

#[repr(transparent)]
pub struct RawValues<'ctx, K, V, P = CoreSliceItemPtrs<K>>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    inner: RawIter<'ctx, K, V, P>,
}

impl<'ctx, K, V, P> RawValues<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    pub(crate) fn from_inner(inner: Inner<'ctx, K, V, P>) -> Self {
        let inner = RawIter::from_inner(inner);
        Self { inner }
    }

    #[inline]
    fn into_inner(self) -> Inner<'ctx, K, V, P> {
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

        let (context, _, values) = inner.as_slice_ptrs_with_context();
        (context, values)
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
    pub fn cast_mut(self) -> RawValuesMut<'ctx, K, V, P> {
        let inner = self.into_inner().cast_mut();
        RawValuesMut::from_inner(inner)
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> Values<'ctx, 'a, K, V, P> {
        let inner = unsafe { self.into_inner().as_ref_unchecked() };
        let inner = Iter::from_inner(inner);
        unsafe { Values::from_inner(inner) }
    }
}

impl<K, V, P> Debug for RawValues<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    for<'ctx> SlicePtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = &self.as_slice_ptrs();
        f.debug_tuple("RawValues").field(slices).finish()
    }
}

impl<K, V, P> Clone for RawValues<'_, K, V, P>
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

impl<'ctx, K, V, P> Iterator for RawValues<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Item = Ptrs<'ctx, V>;

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

impl<K, V, P> DoubleEndedIterator for RawValues<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(_, value)| value)
    }
}

impl<K, V, P> ExactSizeIterator for RawValues<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn len(&self) -> usize {
        RawValues::len(self)
    }
}

impl<K, V, P> FusedIterator for RawValues<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
}
