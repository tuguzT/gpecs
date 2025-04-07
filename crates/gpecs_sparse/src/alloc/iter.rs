use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use crate::{
    pair::{KeyValuePair, KeyValueSlices, KeyValueSlicesMut},
    soa::{traits::Soa, vec},
};

#[repr(transparent)]
pub struct IntoKeys<K, V>
where
    V: Soa,
{
    inner: vec::IntoIter<KeyValuePair<K, V>>,
}

impl<K, V> IntoKeys<K, V>
where
    V: Soa,
{
    #[inline]
    pub(crate) fn new(inner: vec::IntoIter<KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> &[K] {
        let Self { inner } = self;

        let KeyValueSlices { keys, .. } = inner.as_slices();
        keys
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [K] {
        let Self { inner } = self;

        let KeyValueSlicesMut { keys, .. } = inner.as_mut_slices();
        keys
    }
}

impl<K, V> Debug for IntoKeys<K, V>
where
    K: Debug,
    V: Soa,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("IntoKeys").field(keys).finish()
    }
}

impl<K, V> Default for IntoKeys<K, V>
where
    V: Soa,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Clone for IntoKeys<K, V>
where
    V: Soa,
    vec::IntoIter<KeyValuePair<K, V>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> AsRef<[K]> for IntoKeys<K, V>
where
    V: Soa,
{
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<K, V> AsMut<[K]> for IntoKeys<K, V>
where
    V: Soa,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [K] {
        self.as_mut_slice()
    }
}

impl<K, V> Iterator for IntoKeys<K, V>
where
    V: Soa,
{
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|KeyValuePair { key, .. }| key)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, KeyValuePair { key, .. }| f(acc, key))
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V>
where
    V: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|KeyValuePair { key, .. }| key)
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V>
where
    V: Soa,
{
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IntoKeys<K, V> where V: Soa {}

#[repr(transparent)]
pub struct IntoValues<K, V>
where
    V: Soa,
{
    inner: vec::IntoIter<KeyValuePair<K, V>>,
}

impl<K, V> IntoValues<K, V>
where
    V: Soa,
{
    #[inline]
    pub(crate) fn new(inner: vec::IntoIter<KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> V::Slices<'_> {
        let Self { inner } = self;

        let KeyValueSlices { values, .. } = inner.as_slices();
        values
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> V::SlicesMut<'_> {
        let Self { inner } = self;

        let KeyValueSlicesMut { values, .. } = inner.as_mut_slices();
        values
    }
}

impl<K, V> Debug for IntoValues<K, V>
where
    V: Soa,
    for<'a> V::Slices<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("IntoValues").field(values).finish()
    }
}

impl<K, V> Default for IntoValues<K, V>
where
    V: Soa,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Clone for IntoValues<K, V>
where
    V: Soa,
    vec::IntoIter<KeyValuePair<K, V>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, K, V> AsRef<[T]> for IntoValues<K, V>
where
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, K, V> AsMut<[T]> for IntoValues<K, V>
where
    for<'a> V: Soa<SlicesMut<'a> = &'a mut [T]> + 'a,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<K, V> Iterator for IntoValues<K, V>
where
    V: Soa,
{
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|KeyValuePair { value, .. }| value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, KeyValuePair { value, .. }| f(acc, value))
    }
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V>
where
    V: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|KeyValuePair { value, .. }| value)
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V>
where
    V: Soa,
{
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> where V: Soa {}

pub struct IntoIter<K, V>
where
    V: Soa,
{
    inner: vec::IntoIter<KeyValuePair<K, V>>,
}

impl<K, V> IntoIter<K, V>
where
    V: Soa,
{
    #[inline]
    pub(crate) fn new(inner: vec::IntoIter<KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { inner } = self;

        let KeyValueSlices { keys, .. } = inner.as_slices();
        keys
    }

    #[inline]
    pub fn as_keys_mut_slice(&mut self) -> &mut [K] {
        let Self { inner } = self;

        let KeyValueSlicesMut { keys, .. } = inner.as_mut_slices();
        keys
    }

    #[inline]
    pub fn as_values_slice(&self) -> V::Slices<'_> {
        let Self { inner } = self;

        let KeyValueSlices { values, .. } = inner.as_slices();
        values
    }

    #[inline]
    pub fn as_values_mut_slice(&mut self) -> V::SlicesMut<'_> {
        let Self { inner } = self;

        let KeyValueSlicesMut { values, .. } = inner.as_mut_slices();
        values
    }

    #[inline]
    pub fn as_slices(&self) -> (&[K], V::Slices<'_>) {
        let Self { inner } = self;

        let KeyValueSlices { keys, values } = inner.as_slices();
        (keys, values)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [K], V::SlicesMut<'_>) {
        let Self { inner } = self;

        let KeyValueSlicesMut { keys, values } = inner.as_mut_slices();
        (keys, values)
    }
}

impl<K, V> Debug for IntoIter<K, V>
where
    K: Debug,
    V: Soa,
    for<'a> V::Slices<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = self.as_slices();
        f.debug_struct("IntoIter")
            .field("keys", &keys)
            .field("values", &values)
            .finish()
    }
}

impl<K, V> Default for IntoIter<K, V>
where
    V: Soa,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Clone for IntoIter<K, V>
where
    V: Soa,
    vec::IntoIter<KeyValuePair<K, V>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, K, V> AsRef<[T]> for IntoIter<K, V>
where
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<T, K, V> AsMut<[T]> for IntoIter<K, V>
where
    for<'a> V: Soa<SlicesMut<'a> = &'a mut [T]> + 'a,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_values_mut_slice()
    }
}

impl<K, V> Iterator for IntoIter<K, V>
where
    V: Soa,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(Into::into)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V>
where
    V: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(Into::into)
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V>
where
    V: Soa,
{
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> where V: Soa {}

pub struct Drain<'a, K, V>
where
    V: Soa,
{
    inner: vec::Drain<'a, KeyValuePair<K, V>>,
}

impl<'a, K, V> Drain<'a, K, V>
where
    V: Soa,
{
    #[inline]
    pub(crate) fn new(inner: vec::Drain<'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { inner } = self;

        let KeyValueSlices { keys, .. } = inner.as_slices();
        keys
    }

    #[inline]
    pub fn as_values_slice(&self) -> V::Slices<'_> {
        let Self { inner } = self;

        let KeyValueSlices { values, .. } = inner.as_slices();
        values
    }
}

impl<K, V> Debug for Drain<'_, K, V>
where
    K: Debug,
    V: Soa,
    for<'a> V::Slices<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_keys_slice();
        let values = &self.as_values_slice();
        f.debug_struct("Drain")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<T, K, V> AsRef<[T]> for Drain<'_, K, V>
where
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<K, V> Iterator for Drain<'_, K, V>
where
    V: Soa,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(Into::into)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for Drain<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(Into::into)
    }
}

impl<K, V> ExactSizeIterator for Drain<'_, K, V>
where
    V: Soa,
{
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Drain<'_, K, V> where V: Soa {}
