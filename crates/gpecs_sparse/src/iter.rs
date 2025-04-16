use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

#[cfg(feature = "alloc")]
pub use crate::alloc::iter::{Drain, IntoIter, IntoKeys, IntoValues};

use crate::{
    pair::{KeyValuePair, KeyValueRefs, KeyValueRefsMut, KeyValueSlices, KeyValueSlicesMut},
    soa::{slice, traits::Soa},
};

#[repr(transparent)]
pub struct Keys<'a, K, V>
where
    V: Soa,
{
    inner: slice::Iter<'a, KeyValuePair<K, V>>,
}

impl<'a, K, V> Keys<'a, K, V>
where
    V: Soa,
{
    #[inline]
    pub(crate) fn new(inner: slice::Iter<'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [K] {
        let Self { inner } = self;

        let KeyValueSlices { keys, .. } = inner.as_slices();
        keys
    }
}

impl<K, V> Debug for Keys<'_, K, V>
where
    K: Debug,
    V: Soa,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("Keys").field(keys).finish()
    }
}

impl<K, V> Clone for Keys<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> AsRef<[K]> for Keys<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V>
where
    V: Soa,
{
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|KeyValueRefs { key, .. }| key)
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
        inner.last().map(|KeyValueRefs { key, .. }| key)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|KeyValueRefs { key, .. }| key)
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|KeyValueRefs { key, .. }| f(key))
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, KeyValueRefs { key, .. }| f(acc, key))
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.all(|KeyValueRefs { key, .. }| f(key))
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.any(|KeyValueRefs { key, .. }| f(key))
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .find(|KeyValueRefs { key, .. }| predicate(key))
            .map(|KeyValueRefs { key, .. }| key)
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.find_map(|KeyValueRefs { key, .. }| f(key))
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.position(|KeyValueRefs { key, .. }| predicate(key))
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.rposition(|KeyValueRefs { key, .. }| predicate(key))
    }
}

impl<K, V> DoubleEndedIterator for Keys<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|KeyValueRefs { key, .. }| key)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|KeyValueRefs { key, .. }| key)
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Keys<'_, K, V> where V: Soa {}

#[repr(transparent)]
pub struct Values<'a, K, V>
where
    V: Soa,
{
    inner: slice::Iter<'a, KeyValuePair<K, V>>,
}

impl<'a, K, V> Values<'a, K, V>
where
    V: Soa,
{
    #[inline]
    pub(crate) fn new(inner: slice::Iter<'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> V::Slices<'a> {
        let Self { inner } = self;

        let KeyValueSlices { values, .. } = inner.as_slices();
        values
    }
}

impl<K, V> Debug for Values<'_, K, V>
where
    V: Soa,
    for<'a> V::Slices<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("Values").field(values).finish()
    }
}

impl<K, V> Clone for Values<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, K, V> AsRef<[T]> for Values<'_, K, V>
where
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V>
where
    V: Soa,
{
    type Item = V::Refs<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|KeyValueRefs { value, .. }| value)
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
        inner.last().map(|KeyValueRefs { value, .. }| value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|KeyValueRefs { value, .. }| value)
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|KeyValueRefs { value, .. }| f(value))
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, KeyValueRefs { value, .. }| f(acc, value))
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.all(|KeyValueRefs { value, .. }| f(value))
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.any(|KeyValueRefs { value, .. }| f(value))
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .find(|KeyValueRefs { value, .. }| predicate(value))
            .map(|KeyValueRefs { value, .. }| value)
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.find_map(|KeyValueRefs { value, .. }| f(value))
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.position(|KeyValueRefs { value, .. }| predicate(value))
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.rposition(|KeyValueRefs { value, .. }| predicate(value))
    }
}

impl<K, V> DoubleEndedIterator for Values<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|KeyValueRefs { value, .. }| value)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|KeyValueRefs { value, .. }| value)
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> where V: Soa {}

#[repr(transparent)]
pub struct ValuesMut<'a, K, V>
where
    V: Soa,
{
    inner: slice::IterMut<'a, KeyValuePair<K, V>>,
}

impl<'a, K, V> ValuesMut<'a, K, V>
where
    V: Soa,
{
    #[inline]
    pub(crate) fn new(inner: slice::IterMut<'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_slice(self) -> V::SlicesMut<'a> {
        let Self { inner } = self;

        let KeyValueSlicesMut { values, .. } = inner.into_slices();
        values
    }

    #[inline]
    pub fn as_slice(&self) -> V::Slices<'_> {
        let Self { inner } = self;

        let KeyValueSlices { values, .. } = inner.as_slices();
        values
    }
}

impl<K, V> Debug for ValuesMut<'_, K, V>
where
    V: Soa,
    for<'a> V::Slices<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("ValuesMut").field(values).finish()
    }
}

impl<T, K, V> AsRef<[T]> for ValuesMut<'_, K, V>
where
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V>
where
    V: Soa,
{
    type Item = V::RefsMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|KeyValueRefsMut { value, .. }| value)
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
        inner.last().map(|KeyValueRefsMut { value, .. }| value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|KeyValueRefsMut { value, .. }| value)
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|KeyValueRefsMut { value, .. }| f(value))
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, KeyValueRefsMut { value, .. }| f(acc, value))
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.all(|KeyValueRefsMut { value, .. }| f(value))
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.any(|KeyValueRefsMut { value, .. }| f(value))
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .find(|KeyValueRefsMut { value, .. }| predicate(value))
            .map(|KeyValueRefsMut { value, .. }| value)
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.find_map(|KeyValueRefsMut { value, .. }| f(value))
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.position(|KeyValueRefsMut { value, .. }| predicate(value))
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.rposition(|KeyValueRefsMut { value, .. }| predicate(value))
    }
}

impl<K, V> DoubleEndedIterator for ValuesMut<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|KeyValueRefsMut { value, .. }| value)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|KeyValueRefsMut { value, .. }| value)
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for ValuesMut<'_, K, V> where V: Soa {}

pub struct Iter<'a, K, V>
where
    V: Soa,
{
    inner: slice::Iter<'a, KeyValuePair<K, V>>,
}

impl<'a, K, V> Iter<'a, K, V>
where
    V: Soa,
{
    #[inline]
    pub(crate) fn new(inner: slice::Iter<'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &'a [K] {
        let Self { inner } = self;

        let KeyValueSlices { keys, .. } = inner.as_slices();
        keys
    }

    #[inline]
    pub fn as_values_slice(&self) -> V::Slices<'a> {
        let Self { inner } = self;

        let KeyValueSlices { values, .. } = inner.as_slices();
        values
    }

    #[inline]
    pub fn as_slices(&self) -> (&'a [K], V::Slices<'a>) {
        let Self { inner } = self;

        let KeyValueSlices { keys, values } = inner.as_slices();
        (keys, values)
    }
}

impl<K, V> Debug for Iter<'_, K, V>
where
    K: Debug,
    V: Soa,
    for<'a> V::Slices<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = self.as_slices();
        f.debug_struct("Iter")
            .field("keys", &keys)
            .field("values", &values)
            .finish()
    }
}

impl<K, V> Clone for Iter<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<T, K, V> AsRef<[T]> for Iter<'_, K, V>
where
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    V: Soa,
{
    type Item = (&'a K, V::Refs<'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|KeyValueRefs { key, value }| (key, value))
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
        inner.last().map(|KeyValueRefs { key, value }| (key, value))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|KeyValueRefs { key, value }| (key, value))
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|KeyValueRefs { key, value }| f((key, value)))
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|KeyValueRefs { key, value }| (key, value))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|KeyValueRefs { key, value }| (key, value))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> where V: Soa {}

pub struct IterMut<'a, K, V>
where
    V: Soa,
{
    inner: slice::IterMut<'a, KeyValuePair<K, V>>,
}

impl<'a, K, V> IterMut<'a, K, V>
where
    V: Soa,
{
    #[inline]
    pub(crate) fn new(inner: slice::IterMut<'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_keys_slice(self) -> &'a [K] {
        let Self { inner } = self;

        let KeyValueSlicesMut { keys, .. } = inner.into_slices();
        keys
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { inner } = self;

        let KeyValueSlices { keys, .. } = inner.as_slices();
        keys
    }

    #[inline]
    pub fn into_values_slice(self) -> V::SlicesMut<'a> {
        let Self { inner } = self;

        let KeyValueSlicesMut { values, .. } = inner.into_slices();
        values
    }

    #[inline]
    pub fn as_values_slice(&self) -> V::Slices<'_> {
        let Self { inner } = self;

        let KeyValueSlices { values, .. } = inner.as_slices();
        values
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [K], V::SlicesMut<'a>) {
        let Self { inner } = self;

        let KeyValueSlicesMut { keys, values } = inner.into_slices();
        (keys, values)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[K], V::Slices<'_>) {
        let Self { inner } = self;

        let KeyValueSlices { keys, values } = inner.as_slices();
        (keys, values)
    }
}

impl<K, V> Debug for IterMut<'_, K, V>
where
    K: Debug,
    V: Soa,
    for<'a> V::Slices<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = self.as_slices();
        f.debug_struct("IterMut")
            .field("keys", &keys)
            .field("values", &values)
            .finish()
    }
}

impl<T, K, V> AsRef<[T]> for IterMut<'_, K, V>
where
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    V: Soa,
{
    type Item = (&'a K, V::RefsMut<'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next()
            .map(|KeyValueRefsMut { key, value }| (&*key, value))
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
        inner
            .last()
            .map(|KeyValueRefsMut { key, value }| (&*key, value))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth(n)
            .map(|KeyValueRefsMut { key, value }| (&*key, value))
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|KeyValueRefsMut { key, value }| f((&*key, value)))
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|KeyValueRefsMut { key, value }| (&*key, value))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|KeyValueRefsMut { key, value }| (&*key, value))
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V>
where
    V: Soa,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> where V: Soa {}
