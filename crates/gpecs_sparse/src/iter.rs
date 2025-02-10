use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_soa::{slice, vec};

#[repr(transparent)]
pub struct Keys<'a, K, V> {
    inner: slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Keys<'a, K, V> {
    #[inline]
    pub(crate) fn new(inner: slice::Iter<'a, (K, V)>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices();
        keys
    }
}

impl<K, V> Debug for Keys<'_, K, V>
where
    K: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("Keys").field(keys).finish()
    }
}

impl<K, V> Default for Keys<'_, K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Clone for Keys<'_, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> AsRef<[K]> for Keys<'_, K, V> {
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(key, _)| key)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner: keys } = self;
        keys.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner: keys } = self;
        keys.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|(key, _)| key)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(key, _)| key)
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|(key, _)| f(key))
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, (key, _)| f(acc, key))
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.all(|(key, _)| f(key))
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.any(|(key, _)| f(key))
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.find(|(key, _)| predicate(key)).map(|(key, _)| key)
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.find_map(|(key, _)| f(key))
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.position(|(key, _)| predicate(key))
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.rposition(|(key, _)| predicate(key))
    }
}

impl<K, V> DoubleEndedIterator for Keys<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(key, _)| key)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|(key, _)| key)
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Keys<'_, K, V> {}

#[repr(transparent)]
pub struct IntoKeys<K, V> {
    inner: vec::IntoIter<(K, V)>,
}

impl<K, V> IntoKeys<K, V> {
    #[inline]
    pub(crate) fn new(inner: vec::IntoIter<(K, V)>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> &[K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices();
        keys
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_mut_slices();
        keys
    }
}

impl<K, V> Debug for IntoKeys<K, V>
where
    K: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("IntoKeys").field(keys).finish()
    }
}

impl<K, V> Default for IntoKeys<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Clone for IntoKeys<K, V>
where
    vec::IntoIter<(K, V)>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> AsRef<[K]> for IntoKeys<K, V> {
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<K, V> AsMut<[K]> for IntoKeys<K, V> {
    #[inline]
    fn as_mut(&mut self) -> &mut [K] {
        self.as_mut_slice()
    }
}

impl<K, V> Iterator for IntoKeys<K, V> {
    type Item = K;

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
        inner.fold(init, |acc, (key, _)| f(acc, key))
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(key, _)| key)
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V> {
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IntoKeys<K, V> {}

#[repr(transparent)]
pub struct Values<'a, K, V> {
    inner: slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Values<'a, K, V> {
    #[inline]
    pub(crate) fn new(inner: slice::Iter<'a, (K, V)>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [V] {
        let Self { inner } = self;

        let (_, values) = inner.as_slices();
        values
    }
}

impl<K, V> Debug for Values<'_, K, V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("Values").field(values).finish()
    }
}

impl<K, V> Default for Values<'_, K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Clone for Values<'_, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> AsRef<[V]> for Values<'_, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

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

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|(_, value)| value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(_, value)| value)
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|(_, value)| f(value))
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, (_, value)| f(acc, value))
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.all(|(_, value)| f(value))
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.any(|(_, value)| f(value))
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .find(|(_, value)| predicate(value))
            .map(|(_, value)| value)
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.find_map(|(_, value)| f(value))
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.position(|(_, value)| predicate(value))
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.rposition(|(_, value)| predicate(value))
    }
}

impl<K, V> DoubleEndedIterator for Values<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(_, value)| value)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|(_, value)| value)
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> {}

#[repr(transparent)]
pub struct ValuesMut<'a, K, V> {
    inner: slice::IterMut<'a, (K, V)>,
}

impl<'a, K, V> ValuesMut<'a, K, V> {
    #[inline]
    pub(crate) fn new(inner: slice::IterMut<'a, (K, V)>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_slice(self) -> &'a [V] {
        let Self { inner } = self;

        let (_, values) = inner.into_slices();
        values
    }

    #[inline]
    pub fn as_slice(&self) -> &[V] {
        let Self { inner } = self;

        let (_, values) = inner.as_slices();
        values
    }
}

impl<K, V> Debug for ValuesMut<'_, K, V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("ValuesMut").field(values).finish()
    }
}

impl<K, V> Default for ValuesMut<'_, K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> AsRef<[V]> for ValuesMut<'_, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

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

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|(_, value)| value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(_, value)| value)
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|(_, value)| f(value))
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, (_, value)| f(acc, value))
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.all(|(_, value)| f(value))
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.any(|(_, value)| f(value))
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .find(|(_, value)| predicate(value))
            .map(|(_, value)| value)
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.find_map(|(_, value)| f(value))
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.position(|(_, value)| predicate(value))
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.rposition(|(_, value)| predicate(value))
    }
}

impl<K, V> DoubleEndedIterator for ValuesMut<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(_, value)| value)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|(_, value)| value)
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}

#[repr(transparent)]
pub struct IntoValues<K, V> {
    inner: vec::IntoIter<(K, V)>,
}

impl<K, V> IntoValues<K, V> {
    #[inline]
    pub(crate) fn new(inner: vec::IntoIter<(K, V)>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> &[V] {
        let Self { inner } = self;

        let (_, values) = inner.as_slices();
        values
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [V] {
        let Self { inner } = self;

        let (_, values) = inner.as_mut_slices();
        values
    }
}

impl<K, V> Debug for IntoValues<K, V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("IntoValues").field(values).finish()
    }
}

impl<K, V> Default for IntoValues<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Clone for IntoValues<K, V>
where
    vec::IntoIter<(K, V)>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> AsRef<[V]> for IntoValues<K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<K, V> AsMut<[V]> for IntoValues<K, V> {
    #[inline]
    fn as_mut(&mut self) -> &mut [V] {
        self.as_mut_slice()
    }
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

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
        inner.fold(init, |acc, (_, value)| f(acc, value))
    }
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(_, value)| value)
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> {}

pub struct Iter<'a, K, V> {
    inner: slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Iter<'a, K, V> {
    #[inline]
    pub(crate) fn new(inner: slice::Iter<'a, (K, V)>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &'a [K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices();
        keys
    }

    #[inline]
    pub fn as_values_slice(&self) -> &'a [V] {
        let Self { inner } = self;

        let (_, values) = inner.as_slices();
        values
    }

    #[inline]
    pub fn as_slices(&self) -> (&'a [K], &'a [V]) {
        let Self { inner } = self;
        inner.as_slices()
    }
}

impl<K, V> Debug for Iter<'_, K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let (keys, values) = inner.as_slices();
        f.debug_struct("Iter")
            .field("keys", &keys)
            .field("values", &values)
            .finish()
    }
}

impl<K, V> Default for Iter<'_, K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Clone for Iter<'_, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> AsRef<[V]> for Iter<'_, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_values_slice()
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next()
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
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(f)
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n)
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

pub struct IterMut<'a, K, V> {
    inner: slice::IterMut<'a, (K, V)>,
}

impl<'a, K, V> IterMut<'a, K, V> {
    #[inline]
    pub(crate) fn new(inner: slice::IterMut<'a, (K, V)>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_keys_slice(self) -> &'a [K] {
        let Self { inner } = self;

        let (keys, _) = inner.into_slices();
        keys
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices();
        keys
    }

    #[inline]
    pub fn into_values_slice(self) -> &'a mut [V] {
        let Self { inner } = self;

        let (_, values) = inner.into_slices();
        values
    }

    #[inline]
    pub fn as_values_slice(&self) -> &[V] {
        let Self { inner } = self;

        let (_, values) = inner.as_slices();
        values
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [K], &'a mut [V]) {
        let Self { inner } = self;

        let (keys, values) = inner.into_slices();
        (keys, values)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[K], &[V]) {
        let Self { inner } = self;

        let (keys, values) = inner.as_slices();
        (keys, values)
    }
}

impl<K, V> Debug for IterMut<'_, K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let (keys, values) = inner.as_slices();
        f.debug_struct("IterMut")
            .field("keys", &keys)
            .field("values", &values)
            .finish()
    }
}

impl<K, V> Default for IterMut<'_, K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> AsRef<[V]> for IterMut<'_, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_values_slice()
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(key, value)| (&*key, value))
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
        let Self { inner: keys } = self;
        keys.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(|(key, value)| (&*key, value))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(key, value)| (&*key, value))
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|(key, value)| f((&*key, value)))
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(key, value)| (&*key, value))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|(key, value)| (&*key, value))
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> {}

pub struct IntoIter<K, V> {
    inner: vec::IntoIter<(K, V)>,
}

impl<K, V> IntoIter<K, V> {
    #[inline]
    pub(crate) fn new(inner: vec::IntoIter<(K, V)>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices();
        keys
    }

    #[inline]
    pub fn as_keys_mut_slice(&mut self) -> &mut [K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_mut_slices();
        keys
    }

    #[inline]
    pub fn as_values_slice(&self) -> &[V] {
        let Self { inner } = self;

        let (_, values) = inner.as_slices();
        values
    }

    #[inline]
    pub fn as_values_mut_slice(&mut self) -> &mut [V] {
        let Self { inner } = self;

        let (_, values) = inner.as_mut_slices();
        values
    }

    #[inline]
    pub fn as_slices(&self) -> (&[K], &[V]) {
        let Self { inner } = self;
        inner.as_slices()
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [K], &mut [V]) {
        let Self { inner } = self;
        inner.as_mut_slices()
    }
}

impl<K, V> Debug for IntoIter<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let (keys, values) = inner.as_slices();
        f.debug_struct("IntoIter")
            .field("keys", &keys)
            .field("values", &values)
            .finish()
    }
}

impl<K, V> Default for IntoIter<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Clone for IntoIter<K, V>
where
    vec::IntoIter<(K, V)>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> AsRef<[V]> for IntoIter<K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_values_slice()
    }
}

impl<K, V> AsMut<[V]> for IntoIter<K, V> {
    #[inline]
    fn as_mut(&mut self) -> &mut [V] {
        self.as_values_mut_slice()
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next()
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

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back()
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

pub struct Drain<'a, K, V> {
    inner: vec::Drain<'a, (K, V)>,
}

impl<K, V> Debug for Drain<'_, K, V>
where
    K: Debug,
    V: Debug,
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

impl<'a, K, V> Drain<'a, K, V> {
    #[inline]
    pub(crate) fn new(inner: vec::Drain<'a, (K, V)>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices();
        keys
    }

    #[inline]
    pub fn as_values_slice(&self) -> &[V] {
        let Self { inner } = self;

        let (_, values) = inner.as_slices();
        values
    }
}

impl<K, V> AsRef<[V]> for Drain<'_, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_values_slice()
    }
}

impl<K, V> Iterator for Drain<'_, K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for Drain<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back()
    }
}

impl<K, V> ExactSizeIterator for Drain<'_, K, V> {
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Drain<'_, K, V> {}
