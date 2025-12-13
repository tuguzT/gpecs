use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    slice,
};

#[cfg(feature = "alloc")]
pub use crate::alloc::iter::{Drain, IntoIter, IntoKeys, IntoValues};

use crate::{
    pair::KeyValuePair,
    soa::{
        self,
        traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs, Soa},
    },
};

pub struct RawKeys<'c, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c,
{
    inner: RawIter<'c, K, V>,
}

impl<'c, K, V> RawKeys<'c, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(crate) fn from_inner(inner: soa::slice::RawIter<'c, KeyValuePair<K, V>>) -> Self {
        let inner = RawIter::from_inner(inner);
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
    pub fn as_slice_ptr(&self) -> *const [K] {
        let (_, keys) = self.as_slice_ptr_with_context();
        keys
    }

    #[inline]
    pub fn as_slice_ptr_with_context(&self) -> (&'c V::Context, *const [K]) {
        let Self { inner } = self;

        let (context, keys, _) = inner.as_slice_ptrs_with_context();
        (context, keys)
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> Keys<'c, 'a, K, V> {
        unsafe { Keys::from_inner(self) }
    }
}

impl<K, V> Debug for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice_ptr();
        f.debug_tuple("RawKeys").field(keys).finish()
    }
}

impl<K, V> Clone for RawKeys<'_, K, V>
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

impl<K, V> Iterator for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = *const K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(key, _)| key)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(key, _)| key)
    }
}

impl<K, V> ExactSizeIterator for RawKeys<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        RawKeys::len(self)
    }
}

impl<K, V> FusedIterator for RawKeys<'_, K, V> where V: RawSoa + ?Sized {}

pub struct Keys<'c, 'a, K, V>
where
    K: 'c + 'a,
    V: RawSoa + ?Sized + 'c,
{
    inner: RawKeys<'c, K, V>,
    phantom: PhantomData<&'a ()>,
}

impl<'c, 'a, K, V> Keys<'c, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    unsafe fn from_inner(inner: RawKeys<'c, K, V>) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_raw_keys(self) -> RawKeys<'c, K, V> {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &'c V::Context {
        let Self { inner, .. } = self;
        inner.context()
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [K] {
        let (_, keys) = self.as_slice_with_context();
        keys
    }

    #[inline]
    pub fn as_slice_with_context(&self) -> (&'c V::Context, &'a [K]) {
        let Self { inner, .. } = self;

        let (context, keys) = inner.as_slice_ptr_with_context();
        let keys = unsafe { slice::from_raw_parts(keys.cast(), keys.len()) };
        (context, keys)
    }
}

impl<K, V> Debug for Keys<'_, '_, K, V>
where
    K: Debug,
    V: RawSoa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("Keys").field(keys).finish()
    }
}

impl<K, V> Clone for Keys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;

        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<K, V> AsRef<[K]> for Keys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for Keys<'_, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next().map(|key| unsafe { &*key })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for Keys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next_back().map(|key| unsafe { &*key })
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, '_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Keys::len(self)
    }
}

impl<K, V> FusedIterator for Keys<'_, '_, K, V> where V: RawSoa {}

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
    pub(crate) fn from_inner(inner: soa::slice::RawIter<'c, KeyValuePair<K, V>>) -> Self {
        let inner = RawIter::from_inner(inner);
        Self { inner }
    }

    #[inline]
    fn into_inner(self) -> soa::slice::RawIter<'c, KeyValuePair<K, V>> {
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

pub struct Values<'c, 'a, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c + 'a,
{
    inner: Iter<'c, 'a, K, V>,
}

impl<'c, 'a, K, V> Values<'c, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    unsafe fn from_inner(inner: Iter<'c, 'a, K, V>) -> Self {
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
}

impl<'c, 'a, K, V> Values<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn as_slices(&self) -> V::Slices<'c, 'a> {
        let (_, values) = self.as_slices_with_context();
        values
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&'c V::Context, V::Slices<'c, 'a>) {
        let Self { inner } = self;
        let (context, _, values) = inner.as_slices_with_context();
        (context, values)
    }
}

impl<K, V> Debug for Values<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
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
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Into<&'any [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices().into()
    }
}

impl<'c, 'a, K, V> Iterator for Values<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    type Item = V::Refs<'c, 'a>;

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

impl<K, V> DoubleEndedIterator for Values<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next_back().map(|(_, value)| value)
    }
}

impl<K, V> ExactSizeIterator for Values<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Values::len(self)
    }
}

impl<K, V> FusedIterator for Values<'_, '_, K, V> where V: Soa {}

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
    pub(crate) fn from_inner(inner: soa::slice::RawIterMut<'c, KeyValuePair<K, V>>) -> Self {
        let inner = RawIterMut::from_inner(inner);
        Self { inner }
    }

    #[inline]
    fn into_inner(self) -> soa::slice::RawIterMut<'c, KeyValuePair<K, V>> {
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
    pub unsafe fn deref<'a>(self) -> ValuesMut<'c, 'a, K, V> {
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
    unsafe fn from_inner(inner: IterMut<'c, 'a, K, V>) -> Self {
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

pub struct RawIter<'c, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c,
{
    inner: soa::slice::RawIter<'c, KeyValuePair<K, V>>,
}

impl<'c, K, V> RawIter<'c, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(crate) fn from_inner(inner: soa::slice::RawIter<'c, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    fn into_inner(self) -> soa::slice::RawIter<'c, KeyValuePair<K, V>> {
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
    pub fn context(&self) -> &'c V::Context {
        let Self { inner } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const K, Ptrs<'c, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'c V::Context, *const K, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_ptrs(self) -> (*const K, Ptrs<'c, V>) {
        let (_, key, value) = self.into_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'c V::Context, *const K, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_ptrs_with_context();
        let (key, value) = slices.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [K], SlicePtrs<'c, V>) {
        let (_, keys, values) = self.as_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'c V::Context, *const [K], SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (*const [K], SlicePtrs<'c, V>) {
        let (_, keys, values) = self.into_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'c V::Context, *const [K], SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn cast_mut(self) -> RawIterMut<'c, K, V> {
        let Self { inner } = self;
        let inner = inner.cast_mut();
        RawIterMut::from_inner(inner)
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> Iter<'c, 'a, K, V> {
        let inner = unsafe { self.into_inner().deref() };
        Iter::from_inner(inner)
    }
}

impl<K, V> Debug for RawIter<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slice_ptrs();
        f.debug_struct("RawIter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Clone for RawIter<'_, K, V>
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

impl<'c, K, V> Iterator for RawIter<'c, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = (*const K, Ptrs<'c, V>);

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

impl<K, V> DoubleEndedIterator for RawIter<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(From::from)
    }
}

impl<K, V> ExactSizeIterator for RawIter<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        RawIter::len(self)
    }
}

impl<K, V> FusedIterator for RawIter<'_, K, V> where V: RawSoa + ?Sized {}

pub struct Iter<'c, 'a, K, V>
where
    K: 'c + 'a,
    V: RawSoa + ?Sized + 'c + 'a,
{
    inner: soa::slice::Iter<'c, 'a, KeyValuePair<K, V>>,
}

impl<'c, 'a, K, V> Iter<'c, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from_inner(inner: soa::slice::Iter<'c, 'a, KeyValuePair<K, V>>) -> Self {
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
    pub fn as_ptrs(&self) -> (*const K, Ptrs<'c, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'c V::Context, *const K, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [K], SlicePtrs<'c, V>) {
        let (_, keys, values) = self.as_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'c V::Context, *const [K], SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let (key, value) = slices.into_parts();
        (context, key, value)
    }
}

impl<'c, 'a, K, V> Iter<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn as_slices(&self) -> (&'a [K], V::Slices<'c, 'a>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&'c V::Context, &'a [K], V::Slices<'c, 'a>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }
}

impl<K, V> Debug for Iter<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slices();
        f.debug_struct("Iter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Clone for Iter<'_, '_, K, V>
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

impl<T, K, V> AsRef<[T]> for Iter<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Into<&'any [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        let (_, values) = self.as_slices();
        values.into()
    }
}

impl<'c, 'a, K, V> Iterator for Iter<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::Refs<'c, 'a>);

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

impl<K, V> DoubleEndedIterator for Iter<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(From::from)
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Iter::len(self)
    }
}

impl<K, V> FusedIterator for Iter<'_, '_, K, V> where V: Soa {}

pub struct RawIterMut<'c, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c,
{
    inner: soa::slice::RawIterMut<'c, KeyValuePair<K, V>>,
}

impl<'c, K, V> RawIterMut<'c, K, V>
where
    K: 'c,
    V: RawSoa + ?Sized + 'c,
{
    #[inline]
    pub(crate) fn from_inner(inner: soa::slice::RawIterMut<'c, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    fn into_inner(self) -> soa::slice::RawIterMut<'c, KeyValuePair<K, V>> {
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
    pub fn context(&self) -> &'c V::Context {
        let Self { inner } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const K, Ptrs<'c, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'c V::Context, *const K, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut K, MutPtrs<'c, V>) {
        let (_, key, value) = self.as_mut_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&'c V::Context, *mut K, MutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_mut_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_ptrs(self) -> (*const K, Ptrs<'c, V>) {
        let (_, key, value) = self.into_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'c V::Context, *const K, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_ptrs_with_context();
        let (key, value) = slices.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> (*mut K, MutPtrs<'c, V>) {
        let (_, key, value) = self.into_mut_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'c V::Context, *mut K, MutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_mut_ptrs_with_context();
        let (key, value) = slices.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [K], SlicePtrs<'c, V>) {
        let (_, keys, values) = self.as_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'c V::Context, *const [K], SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_slice_mut_ptrs(&mut self) -> (*const [K], SliceMutPtrs<'c, V>) {
        let (_, keys, values) = self.as_slice_mut_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(
        &mut self,
    ) -> (&'c V::Context, *const [K], SliceMutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slice_mut_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (*const [K], SlicePtrs<'c, V>) {
        let (_, keys, values) = self.into_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'c V::Context, *const [K], SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slice_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn into_slice_mut_ptrs(self) -> (*const [K], SliceMutPtrs<'c, V>) {
        let (_, keys, values) = self.into_slice_mut_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slice_mut_ptrs_with_context(
        self,
    ) -> (&'c V::Context, *const [K], SliceMutPtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slice_mut_ptrs_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn cast_const(self) -> RawIter<'c, K, V> {
        let Self { inner } = self;
        let inner = inner.cast_const();
        RawIter::from_inner(inner)
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> IterMut<'c, 'a, K, V> {
        let inner = unsafe { self.into_inner().deref_mut() };
        IterMut::from_inner(inner)
    }
}

impl<K, V> Debug for RawIterMut<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slice_ptrs();
        f.debug_struct("RawIterMut")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Clone for RawIterMut<'_, K, V>
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

impl<'c, K, V> Iterator for RawIterMut<'c, K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = (*mut K, MutPtrs<'c, V>);

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

impl<K, V> DoubleEndedIterator for RawIterMut<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(From::from)
    }
}

impl<K, V> ExactSizeIterator for RawIterMut<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        RawIterMut::len(self)
    }
}

impl<K, V> FusedIterator for RawIterMut<'_, K, V> where V: RawSoa + ?Sized {}

pub struct IterMut<'c, 'a, K, V>
where
    K: 'c + 'a,
    V: RawSoa + ?Sized + 'c + 'a,
{
    inner: soa::slice::IterMut<'c, 'a, KeyValuePair<K, V>>,
}

impl<'c, 'a, K, V> IterMut<'c, 'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from_inner(inner: soa::slice::IterMut<'c, 'a, KeyValuePair<K, V>>) -> Self {
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
    pub fn as_ptrs(&self) -> (*const K, Ptrs<'c, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'c V::Context, *const K, Ptrs<'c, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [K], SlicePtrs<'c, V>) {
        let (_, keys, values) = self.as_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'c V::Context, *const [K], SlicePtrs<'c, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let (key, value) = slices.into_parts();
        (context, key, value)
    }
}

impl<'c, 'a, K, V> IterMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn into_slices(self) -> (&'a [K], V::SlicesMut<'c, 'a>) {
        let (_, keys, values) = self.into_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'c V::Context, &'a [K], V::SlicesMut<'c, 'a>) {
        let Self { inner } = self;

        let (context, slices) = inner.into_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[K], V::Slices<'_, '_>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&V::Context, &[K], V::Slices<'_, '_>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.as_slices_with_context();
        let (keys, values) = slices.into_parts();
        (context, keys, values)
    }
}

impl<K, V> Debug for IterMut<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
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
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Into<&'any [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        let (_, values) = self.as_slices();
        values.into()
    }
}

impl<'c, 'a, K, V> Iterator for IterMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::RefsMut<'c, 'a>);

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

impl<K, V> DoubleEndedIterator for IterMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
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

impl<K, V> ExactSizeIterator for IterMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<K, V> FusedIterator for IterMut<'_, '_, K, V> where V: Soa {}
