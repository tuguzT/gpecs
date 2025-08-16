use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

#[cfg(feature = "alloc")]
pub use crate::alloc::iter::{Drain, IntoIter, IntoKeys, IntoValues};

use crate::{
    pair::{KeyValuePair, KeyValueRefs, KeyValueRefsMut},
    soa::{slice, traits::Soa},
};

#[repr(transparent)]
pub struct Keys<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    inner: slice::Iter<'c, 'a, KeyValuePair<K, V>>,
}

impl<'c, 'a, K, V> Keys<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(inner: slice::Iter<'c, 'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices().into_parts();
        keys
    }
}

impl<K, V> Debug for Keys<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("Keys").field(keys).finish()
    }
}

impl<K, V> Clone for Keys<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<K, V> AsRef<[K]> for Keys<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for Keys<'_, 'a, K, V>
where
    V: Soa + ?Sized,
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
        inner.for_each(|KeyValueRefs { key, .. }| f(key));
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

impl<K, V> DoubleEndedIterator for Keys<'_, '_, K, V>
where
    V: Soa + ?Sized,
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

impl<K, V> ExactSizeIterator for Keys<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Keys<'_, '_, K, V> where V: Soa {}

#[repr(transparent)]
pub struct Values<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    inner: slice::Iter<'c, 'a, KeyValuePair<K, V>>,
}

impl<'c, 'a, K, V> Values<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(inner: slice::Iter<'c, 'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_slice(&self) -> V::Slices<'c, 'a> {
        let Self { inner } = self;

        let (_, values) = inner.as_slices().into_parts();
        values.into_inner()
    }
}

impl<K, V> Debug for Values<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("Values").field(values).finish()
    }
}

impl<K, V> Clone for Values<'_, '_, K, V>
where
    V: Soa + ?Sized,
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
    for<'c, 'any> V: Soa<Slices<'c, 'any> = &'any [T]> + 'any,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'c, 'a, K, V> Iterator for Values<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    type Item = V::Refs<'c, 'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next()
            .map(|KeyValueRefs { value, .. }| value.into_inner())
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
        inner
            .last()
            .map(|KeyValueRefs { value, .. }| value.into_inner())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth(n)
            .map(|KeyValueRefs { value, .. }| value.into_inner())
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|KeyValueRefs { value, .. }| f(value.into_inner()));
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, KeyValueRefs { value, .. }| {
            f(acc, value.into_inner())
        })
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.all(|KeyValueRefs { value, .. }| f(value.into_inner()))
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.any(|KeyValueRefs { value, .. }| f(value.into_inner()))
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .map(|KeyValueRefs { value, .. }| value.into_inner())
            .find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.find_map(|KeyValueRefs { value, .. }| f(value.into_inner()))
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.position(|KeyValueRefs { value, .. }| predicate(value.into_inner()))
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.rposition(|KeyValueRefs { value, .. }| predicate(value.into_inner()))
    }
}

impl<K, V> DoubleEndedIterator for Values<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|KeyValueRefs { value, .. }| value.into_inner())
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|KeyValueRefs { value, .. }| value.into_inner())
    }
}

impl<K, V> ExactSizeIterator for Values<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Values<'_, '_, K, V> where V: Soa {}

#[repr(transparent)]
pub struct ValuesMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    inner: slice::IterMut<'c, 'a, KeyValuePair<K, V>>,
}

impl<'c, 'a, K, V> ValuesMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(inner: slice::IterMut<'c, 'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_slice(self) -> V::SlicesMut<'c, 'a> {
        let Self { inner } = self;

        let (_, values) = inner.into_slices().into_parts();
        values.into_inner()
    }

    #[inline]
    pub fn as_slice(&self) -> V::Slices<'_, '_> {
        let Self { inner } = self;

        let (_, values) = inner.as_slices().into_parts();
        values.into_inner()
    }
}

impl<K, V> Debug for ValuesMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("ValuesMut").field(values).finish()
    }
}

impl<T, K, V> AsRef<[T]> for ValuesMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V: Soa<Slices<'c, 'any> = &'any [T]> + 'any,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
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
        inner
            .next()
            .map(|KeyValueRefsMut { value, .. }| value.into_inner())
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
        inner
            .last()
            .map(|KeyValueRefsMut { value, .. }| value.into_inner())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth(n)
            .map(|KeyValueRefsMut { value, .. }| value.into_inner())
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|KeyValueRefsMut { value, .. }| f(value.into_inner()));
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, KeyValueRefsMut { value, .. }| {
            f(acc, value.into_inner())
        })
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.all(|KeyValueRefsMut { value, .. }| f(value.into_inner()))
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.any(|KeyValueRefsMut { value, .. }| f(value.into_inner()))
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .map(|KeyValueRefsMut { value, .. }| value.into_inner())
            .find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.find_map(|KeyValueRefsMut { value, .. }| f(value.into_inner()))
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.position(|KeyValueRefsMut { value, .. }| predicate(value.into_inner()))
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.rposition(|KeyValueRefsMut { value, .. }| predicate(value.into_inner()))
    }
}

impl<K, V> DoubleEndedIterator for ValuesMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|KeyValueRefsMut { value, .. }| value.into_inner())
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|KeyValueRefsMut { value, .. }| value.into_inner())
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for ValuesMut<'_, '_, K, V> where V: Soa {}

pub struct Iter<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    inner: slice::Iter<'c, 'a, KeyValuePair<K, V>>,
}

impl<'c, 'a, K, V> Iter<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(inner: slice::Iter<'c, 'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &'a [K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices().into_parts();
        keys
    }

    #[inline]
    pub fn as_values_slice(&self) -> V::Slices<'c, 'a> {
        let Self { inner } = self;

        let (_, values) = inner.as_slices().into_parts();
        values.into_inner()
    }

    #[inline]
    pub fn as_slices(&self) -> (&'a [K], V::Slices<'c, 'a>) {
        let Self { inner } = self;

        let (keys, values) = inner.as_slices().into_parts();
        (keys, values.into_inner())
    }
}

impl<K, V> Debug for Iter<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = self.as_slices();
        f.debug_struct("Iter")
            .field("keys", &keys)
            .field("values", &values)
            .finish()
    }
}

impl<K, V> Clone for Iter<'_, '_, K, V>
where
    V: Soa + ?Sized,
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
    for<'c, 'any> V: Soa<Slices<'c, 'any> = &'any [T]> + 'any,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
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
        inner
            .next()
            .map(|KeyValueRefs { key, value }| (key, value.into_inner()))
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
        inner
            .last()
            .map(|KeyValueRefs { key, value }| (key, value.into_inner()))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth(n)
            .map(|KeyValueRefs { key, value }| (key, value.into_inner()))
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|KeyValueRefs { key, value }| f((key, value.into_inner())));
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|KeyValueRefs { key, value }| (key, value.into_inner()))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|KeyValueRefs { key, value }| (key, value.into_inner()))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Iter<'_, '_, K, V> where V: Soa {}

pub struct IterMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    inner: slice::IterMut<'c, 'a, KeyValuePair<K, V>>,
}

impl<'c, 'a, K, V> IterMut<'c, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub(crate) fn new(inner: slice::IterMut<'c, 'a, KeyValuePair<K, V>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_keys_slice(self) -> &'a [K] {
        let Self { inner } = self;

        let (keys, _) = inner.into_slices().into_parts();
        keys
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { inner } = self;

        let (keys, _) = inner.as_slices().into_parts();
        keys
    }

    #[inline]
    pub fn into_values_slice(self) -> V::SlicesMut<'c, 'a> {
        let Self { inner } = self;

        let (_, values) = inner.into_slices().into_parts();
        values.into_inner()
    }

    #[inline]
    pub fn as_values_slice(&self) -> V::Slices<'_, '_> {
        let Self { inner } = self;

        let (_, values) = inner.as_slices().into_parts();
        values.into_inner()
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [K], V::SlicesMut<'c, 'a>) {
        let Self { inner } = self;

        let (keys, values) = inner.into_slices().into_parts();
        (keys, values.into_inner())
    }

    #[inline]
    pub fn as_slices(&self) -> (&[K], V::Slices<'_, '_>) {
        let Self { inner } = self;

        let (keys, values) = inner.as_slices().into_parts();
        (keys, values.into_inner())
    }
}

impl<K, V> Debug for IterMut<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = self.as_slices();
        f.debug_struct("IterMut")
            .field("keys", &keys)
            .field("values", &values)
            .finish()
    }
}

impl<T, K, V> AsRef<[T]> for IterMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'any> V: Soa<Slices<'c, 'any> = &'any [T]> + 'any,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
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
        inner
            .next()
            .map(|KeyValueRefsMut { key, value }| (&*key, value.into_inner()))
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
            .map(|KeyValueRefsMut { key, value }| (&*key, value.into_inner()))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth(n)
            .map(|KeyValueRefsMut { key, value }| (&*key, value.into_inner()))
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|KeyValueRefsMut { key, value }| f((&*key, value.into_inner())));
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|KeyValueRefsMut { key, value }| (&*key, value.into_inner()))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|KeyValueRefsMut { key, value }| (&*key, value.into_inner()))
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for IterMut<'_, '_, K, V> where V: Soa {}
