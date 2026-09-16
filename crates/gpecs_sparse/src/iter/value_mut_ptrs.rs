use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_ptr::slice::{CoreSliceItemPtrs, SliceItemPtrs};

use crate::{
    item::KeyValuePair,
    iter::{IterMut, IterMutPtrs, ValuePtrs, Values, ValuesMut},
    soa::{
        self,
        traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs},
    },
};

type Inner<'ctx, K, V, P> = soa::slice::IterMutPtrs<'ctx, KeyValuePair<K, V, P>>;

#[repr(transparent)]
pub struct ValueMutPtrs<'ctx, K, V, P = CoreSliceItemPtrs<K>>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    inner: IterMutPtrs<'ctx, K, V, P>,
}

impl<'ctx, K, V, P> ValueMutPtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    pub(crate) fn from_inner(inner: Inner<'ctx, K, V, P>) -> Self {
        let inner = IterMutPtrs::from_inner(inner);
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

        let (context, _, values) = inner.as_slice_ptrs_with_context();
        (context, values)
    }

    #[inline]
    pub fn as_mut_slice_ptrs(&mut self) -> SliceMutPtrs<'ctx, V> {
        let (_, values) = self.as_mut_slice_ptrs_with_context();
        values
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(&mut self) -> (&'ctx V::Context, SliceMutPtrs<'ctx, V>) {
        let Self { inner } = self;

        let (context, _, values) = inner.as_mut_slice_ptrs_with_context();
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
    pub fn cast_const(self) -> ValuePtrs<'ctx, K, V, P> {
        let inner = self.into_inner().cast_const();
        ValuePtrs::from_inner(inner)
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> Values<'ctx, 'a, K, V, P> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ValuesMut<'ctx, 'a, K, V, P> {
        let inner = unsafe { self.into_inner().as_mut_unchecked() };
        let inner = IterMut::from_inner(inner);
        unsafe { ValuesMut::from_inner(inner) }
    }
}

impl<K, V, P> Debug for ValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    for<'ctx> SlicePtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = &self.as_slice_ptrs();
        f.debug_tuple("ValueMutPtrs").field(slices).finish()
    }
}

impl<K, V, P> Clone for ValueMutPtrs<'_, K, V, P>
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

impl<'ctx, K, V, P> Iterator for ValueMutPtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Item = MutPtrs<'ctx, V>;

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

impl<K, V, P> DoubleEndedIterator for ValueMutPtrs<'_, K, V, P>
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

impl<K, V, P> ExactSizeIterator for ValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn len(&self) -> usize {
        ValueMutPtrs::len(self)
    }
}

impl<K, V, P> FusedIterator for ValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
}
