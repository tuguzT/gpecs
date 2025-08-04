use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use gpecs_soa::traits::SoaRead;

use crate::{
    pair::KeyValuePair,
    soa::{traits::Soa, vec},
};

#[repr(transparent)]
pub struct IntoKeys<K, V>
where
    V: Soa + ?Sized,
{
    inner: core_alloc::vec::IntoIter<K>,
    phantom: PhantomData<fn() -> V>,
}

impl<K, V> IntoKeys<K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(inner: core_alloc::vec::IntoIter<K>) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[K] {
        let Self { inner, .. } = self;
        inner.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [K] {
        let Self { inner, .. } = self;
        inner.as_mut_slice()
    }
}

impl<K, V> Debug for IntoKeys<K, V>
where
    K: Debug,
    V: Soa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("IntoKeys").field(keys).finish()
    }
}

impl<K, V> Default for IntoKeys<K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<K, V> Clone for IntoKeys<K, V>
where
    K: Clone,
    V: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<K, V> AsRef<[K]> for IntoKeys<K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<K, V> AsMut<[K]> for IntoKeys<K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [K] {
        self.as_mut_slice()
    }
}

impl<K, V> Iterator for IntoKeys<K, V>
where
    V: Soa + ?Sized,
{
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner, .. } = self;
        inner.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner, .. } = self;
        inner.fold(init, f)
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next_back()
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V>
where
    V: Soa + ?Sized,
{
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IntoKeys<K, V> where V: Soa {}

#[repr(transparent)]
pub struct IntoValues<K, V>
where
    V: Soa + ?Sized,
{
    inner: vec::IntoIter<KeyValuePair<K, V>>,
}

impl<K, V> IntoValues<K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(inner: vec::IntoIter<KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> V::Slices<'_, '_> {
        let Self { inner } = self;

        let (_, values) = inner.as_slices().into_parts();
        values
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> V::SlicesMut<'_, '_> {
        let Self { inner } = self;

        let (_, values) = inner.as_mut_slices().into_parts();
        values
    }
}

impl<K, V> Debug for IntoValues<K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("IntoValues").field(values).finish()
    }
}

impl<K, V> Default for IntoValues<K, V>
where
    V: Soa + ?Sized,
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
    V: Soa + ?Sized,
    vec::IntoIter<KeyValuePair<K, V>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<T, K, V> AsRef<[T]> for IntoValues<K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V: Soa<Slices<'c, 'any> = &'any [T]> + 'any,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, K, V> AsMut<[T]> for IntoValues<K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V: Soa<SlicesMut<'c, 'any> = &'any mut [T]> + 'any,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<K, V> Iterator for IntoValues<K, V>
where
    V: SoaRead,
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
    V: SoaRead,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|KeyValuePair { value, .. }| value)
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V>
where
    V: SoaRead,
{
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> where V: SoaRead {}

pub struct IntoIter<K, V>
where
    V: Soa + ?Sized,
{
    inner: vec::IntoIter<KeyValuePair<K, V>>,
}

impl<K, V> IntoIter<K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(inner: vec::IntoIter<KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices().into_parts();
        keys
    }

    #[inline]
    pub fn as_keys_mut_slice(&mut self) -> &mut [K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_mut_slices().into_parts();
        keys
    }

    #[inline]
    pub fn as_values_slice(&self) -> V::Slices<'_, '_> {
        let Self { inner } = self;

        let (_, values) = inner.as_slices().into_parts();
        values
    }

    #[inline]
    pub fn as_values_mut_slice(&mut self) -> V::SlicesMut<'_, '_> {
        let Self { inner } = self;

        let (_, values) = inner.as_mut_slices().into_parts();
        values
    }

    #[inline]
    pub fn as_slices(&self) -> (&[K], V::Slices<'_, '_>) {
        let Self { inner } = self;

        let (keys, values) = inner.as_slices().into_parts();
        (keys, values)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [K], V::SlicesMut<'_, '_>) {
        let Self { inner } = self;
        inner.as_mut_slices().into_parts()
    }
}

impl<K, V> Debug for IntoIter<K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
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
    V: Soa + ?Sized,
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
    V: Soa + ?Sized,
    vec::IntoIter<KeyValuePair<K, V>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<T, K, V> AsRef<[T]> for IntoIter<K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V: Soa<Slices<'c, 'any> = &'any [T]> + 'any,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<T, K, V> AsMut<[T]> for IntoIter<K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V: Soa<SlicesMut<'c, 'any> = &'any mut [T]> + 'any,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_values_mut_slice()
    }
}

impl<K, V> Iterator for IntoIter<K, V>
where
    V: SoaRead,
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
    V: SoaRead,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(Into::into)
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V>
where
    V: SoaRead,
{
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> where V: SoaRead {}

pub struct Drain<'a, K, V>
where
    V: Soa + ?Sized,
{
    inner: vec::Drain<'a, KeyValuePair<K, V>>,
}

impl<'a, K, V> Drain<'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(inner: vec::Drain<'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices().into_parts();
        keys
    }

    #[inline]
    pub fn as_values_slice(&self) -> V::Slices<'_, '_> {
        let Self { inner } = self;

        let (_, values) = inner.as_slices().into_parts();
        values
    }
}

impl<K, V> Debug for Drain<'_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
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
    V: Soa + ?Sized,
    for<'c, 'any> V: Soa<Slices<'c, 'any> = &'any [T]> + 'any,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<K, V> Iterator for Drain<'_, K, V>
where
    V: SoaRead,
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
    V: SoaRead,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(Into::into)
    }
}

impl<K, V> ExactSizeIterator for Drain<'_, K, V>
where
    V: SoaRead,
{
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Drain<'_, K, V> where V: SoaRead {}
