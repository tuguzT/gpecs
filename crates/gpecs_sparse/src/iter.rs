use alloc::vec;
use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    slice,
};

use crate::{check_kv_same_len, match_kv_same_kind};

#[repr(transparent)]
pub struct Keys<'a, K, V> {
    keys: slice::Iter<'a, K>,
    values: PhantomData<&'a V>,
}

impl<'a, K, V> Keys<'a, K, V> {
    #[inline]
    pub(crate) fn new(keys: slice::Iter<'a, K>) -> Self {
        let values = PhantomData;
        Self { keys, values }
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [K] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }
}

impl<'a, K, V> Debug for Keys<'a, K, V>
where
    K: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("Keys").field(keys).finish()
    }
}

impl<'a, K, V> Default for Keys<'a, K, V> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Default::default();
        Self { keys, values }
    }
}

impl<'a, K, V> Clone for Keys<'a, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, values } = self;

        let keys = keys.clone();
        let values = *values;
        Self { keys, values }
    }
}

impl<'a, K, V> AsRef<[K]> for Keys<'a, K, V> {
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, .. } = self;
        keys.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.nth(n)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { keys, .. } = self;
        keys.for_each(f)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { keys, .. } = self;
        keys.fold(init, f)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { keys, .. } = self;
        keys.find_map(f)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.position(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.rposition(predicate)
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.nth_back(n)
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { keys, .. } = self;
        keys.len()
    }
}

impl<'a, K, V> FusedIterator for Keys<'a, K, V> {}

#[repr(transparent)]
pub struct IntoKeys<K, V> {
    keys: vec::IntoIter<K>,
    values: PhantomData<V>,
}

impl<K, V> IntoKeys<K, V> {
    #[inline]
    pub(crate) fn new(keys: vec::IntoIter<K>) -> Self {
        let values = PhantomData;
        Self { keys, values }
    }

    #[inline]
    pub fn as_slice(&self) -> &[K] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [K] {
        let Self { keys, .. } = self;
        keys.as_mut_slice()
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
        let keys = Default::default();
        let values = Default::default();
        Self { keys, values }
    }
}

impl<K, V> Clone for IntoKeys<K, V>
where
    K: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, values } = self;

        let keys = keys.clone();
        let values = *values;
        Self { keys, values }
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
        let Self { keys, .. } = self;
        keys.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, .. } = self;
        keys.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { keys, .. } = self;
        keys.fold(init, f)
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.next_back()
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V> {}

impl<K, V> FusedIterator for IntoKeys<K, V> {}

#[repr(transparent)]
pub struct Values<'a, K, V> {
    keys: PhantomData<&'a K>,
    values: slice::Iter<'a, V>,
}

impl<'a, K, V> Values<'a, K, V> {
    #[inline]
    pub(crate) fn new(values: slice::Iter<'a, V>) -> Self {
        let keys = PhantomData;
        Self { keys, values }
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [V] {
        let Self { values, .. } = self;
        values.as_slice()
    }
}

impl<'a, K, V> Debug for Values<'a, K, V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("Values").field(values).finish()
    }
}

impl<'a, K, V> Default for Values<'a, K, V> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Default::default();
        Self { keys, values }
    }
}

impl<'a, K, V> Clone for Values<'a, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, values } = self;

        let keys = *keys;
        let values = values.clone();
        Self { keys, values }
    }
}

impl<'a, K, V> AsRef<[V]> for Values<'a, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { values, .. } = self;
        values.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { values, .. } = self;
        values.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { values, .. } = self;
        values.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { values, .. } = self;
        values.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values, .. } = self;
        values.nth(n)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { values, .. } = self;
        values.for_each(f)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { values, .. } = self;
        values.fold(init, f)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { values, .. } = self;
        values.find_map(f)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.position(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.rposition(predicate)
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { values, .. } = self;
        values.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values, .. } = self;
        values.nth_back(n)
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { values, .. } = self;
        values.len()
    }
}

impl<'a, K, V> FusedIterator for Values<'a, K, V> {}

#[repr(transparent)]
pub struct ValuesMut<'a, K, V> {
    keys: PhantomData<&'a K>,
    values: slice::IterMut<'a, V>,
}

impl<'a, K, V> ValuesMut<'a, K, V> {
    #[inline]
    pub(crate) fn new(values: slice::IterMut<'a, V>) -> Self {
        let keys = PhantomData;
        Self { keys, values }
    }

    #[inline]
    pub fn into_slice(self) -> &'a [V] {
        let Self { values, .. } = self;
        values.into_slice()
    }

    #[inline]
    pub fn as_slice(&self) -> &[V] {
        let Self { values, .. } = self;
        values.as_slice()
    }
}

impl<'a, K, V> Debug for ValuesMut<'a, K, V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("ValuesMut").field(values).finish()
    }
}

impl<'a, K, V> Default for ValuesMut<'a, K, V> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Default::default();
        Self { values, keys }
    }
}

impl<'a, K, V> AsRef<[V]> for ValuesMut<'a, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { values, .. } = self;
        values.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { values, .. } = self;
        values.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { values, .. } = self;
        values.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { values, .. } = self;
        values.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values, .. } = self;
        values.nth(n)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { values, .. } = self;
        values.for_each(f)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { values, .. } = self;
        values.fold(init, f)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { values, .. } = self;
        values.find_map(f)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.position(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values, .. } = self;
        values.rposition(predicate)
    }
}

impl<'a, K, V> DoubleEndedIterator for ValuesMut<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { values, .. } = self;
        values.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values, .. } = self;
        values.nth_back(n)
    }
}

impl<'a, K, V> ExactSizeIterator for ValuesMut<'a, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { values, .. } = self;
        values.len()
    }
}

impl<'a, K, V> FusedIterator for ValuesMut<'a, K, V> {}

#[derive(Clone)]
#[repr(transparent)]
pub struct IntoValues<K, V> {
    keys: PhantomData<K>,
    values: vec::IntoIter<V>,
}

impl<K, V> IntoValues<K, V> {
    #[inline]
    pub(crate) fn new(values: vec::IntoIter<V>) -> Self {
        let keys = PhantomData;
        Self { keys, values }
    }

    #[inline]
    pub fn as_slice(&self) -> &[V] {
        let Self { values, .. } = self;
        values.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [V] {
        let Self { values, .. } = self;
        values.as_mut_slice()
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
        let keys = Default::default();
        let values = Default::default();
        Self { values, keys }
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
        let Self { values, .. } = self;
        values.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { values, .. } = self;
        values.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { values, .. } = self;
        values.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { values, .. } = self;
        values.fold(init, f)
    }
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { values, .. } = self;
        values.next_back()
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {}

impl<K, V> FusedIterator for IntoValues<K, V> {}

pub struct Iter<'a, K, V> {
    keys: slice::Iter<'a, K>,
    values: slice::Iter<'a, V>,
}

impl<'a, K, V> Iter<'a, K, V> {
    #[inline]
    pub(crate) fn new(keys: slice::Iter<'a, K>, values: slice::Iter<'a, V>) -> Self {
        check_kv_same_len(keys.len(), values.len());
        Self { keys, values }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &'a [K] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_values_slice(&self) -> &'a [V] {
        let Self { values, .. } = self;
        values.as_slice()
    }

    #[inline]
    pub fn as_slices(&self) -> (&'a [K], &'a [V]) {
        let Self { keys, values } = self;
        (keys.as_slice(), values.as_slice())
    }
}

impl<'a, K, V> Debug for Iter<'a, K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;

        let keys = &keys.as_slice();
        let values = &values.as_slice();
        f.debug_struct("Iter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'a, K, V> Default for Iter<'a, K, V> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Default::default();
        Self { keys, values }
    }
}

impl<'a, K, V> Clone for Iter<'a, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, values } = self;

        let keys = keys.clone();
        let values = values.clone();
        Self { keys, values }
    }
}

impl<'a, K, V> AsRef<[V]> for Iter<'a, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_values_slice()
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next();
        let value = values.next();
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, .. } = self;
        keys.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { keys, values } = self;

        let key = keys.last();
        let value = values.last();
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.nth(n);
        let value = values.nth(n);
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        for x in self {
            f(x);
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next_back();
        let value = values.next_back();
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.nth_back(n);
        let value = values.nth_back(n);
        match_kv_same_kind(key, value)
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { keys, .. } = self;
        keys.len()
    }
}

impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}

pub struct IterMut<'a, K, V> {
    keys: slice::Iter<'a, K>,
    values: slice::IterMut<'a, V>,
}

impl<'a, K, V> IterMut<'a, K, V> {
    #[inline]
    pub(crate) fn new(keys: slice::Iter<'a, K>, values: slice::IterMut<'a, V>) -> Self {
        check_kv_same_len(keys.len(), values.len());
        Self { keys, values }
    }

    #[inline]
    pub fn into_keys_slice(self) -> &'a [K] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &'a [K] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn into_values_slice(self) -> &'a mut [V] {
        let Self { values, .. } = self;
        values.into_slice()
    }

    #[inline]
    pub fn as_values_slice(&self) -> &[V] {
        let Self { values, .. } = self;
        values.as_slice()
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [K], &'a mut [V]) {
        let Self { keys, values } = self;
        (keys.as_slice(), values.into_slice())
    }

    #[inline]
    pub fn as_slices(&self) -> (&'a [K], &[V]) {
        let Self { keys, values } = self;
        (keys.as_slice(), values.as_slice())
    }
}

impl<'a, K, V> Debug for IterMut<'a, K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;

        let keys = &keys.as_slice();
        let values = &values.as_slice();
        f.debug_struct("IterMut")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'a, K, V> Default for IterMut<'a, K, V> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Default::default();
        Self { keys, values }
    }
}

impl<'a, K, V> AsRef<[V]> for IterMut<'a, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_values_slice()
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next();
        let value = values.next();
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, .. } = self;
        keys.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { keys, values } = self;

        let key = keys.last();
        let value = values.last();
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.nth(n);
        let value = values.nth(n);
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        for x in self {
            f(x);
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next_back();
        let value = values.next_back();
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.nth_back(n);
        let value = values.nth_back(n);
        match_kv_same_kind(key, value)
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {
    #[inline]
    fn len(&self) -> usize {
        let Self { keys, .. } = self;
        keys.len()
    }
}

impl<'a, K, V> FusedIterator for IterMut<'a, K, V> {}

#[derive(Clone)]
pub struct IntoIter<K, V> {
    keys: vec::IntoIter<K>,
    values: vec::IntoIter<V>,
}

impl<K, V> IntoIter<K, V> {
    #[inline]
    pub(crate) fn new(keys: vec::IntoIter<K>, values: vec::IntoIter<V>) -> Self {
        check_kv_same_len(keys.len(), values.len());
        Self { keys, values }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_keys_mut_slice(&mut self) -> &mut [K] {
        let Self { keys, .. } = self;
        keys.as_mut_slice()
    }

    #[inline]
    pub fn as_values_slice(&self) -> &[V] {
        let Self { values, .. } = self;
        values.as_slice()
    }

    #[inline]
    pub fn as_values_mut_slice(&mut self) -> &mut [V] {
        let Self { values, .. } = self;
        values.as_mut_slice()
    }

    #[inline]
    pub fn as_slices(&self) -> (&[K], &[V]) {
        let Self { keys, values } = self;
        (keys.as_slice(), values.as_slice())
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [K], &mut [V]) {
        let Self { keys, values } = self;
        (keys.as_mut_slice(), values.as_mut_slice())
    }
}

impl<K, V> Debug for IntoIter<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;

        let keys = &keys.as_slice();
        let values = &values.as_slice();
        f.debug_struct("IntoIter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Default for IntoIter<K, V> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Default::default();
        Self { keys, values }
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
        let Self { keys, values } = self;

        let key = keys.next();
        let value = values.next();
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, .. } = self;
        keys.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.count()
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next_back();
        let value = values.next_back();
        match_kv_same_kind(key, value)
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {}

impl<K, V> FusedIterator for IntoIter<K, V> {}

pub struct Drain<'a, K, V> {
    keys: vec::Drain<'a, K>,
    values: vec::Drain<'a, V>,
}

impl<'a, K, V> Debug for Drain<'a, K, V>
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
    pub(crate) fn new(keys: vec::Drain<'a, K>, values: vec::Drain<'a, V>) -> Self {
        check_kv_same_len(keys.len(), values.len());
        Self { keys, values }
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_values_slice(&self) -> &[V] {
        let Self { values, .. } = self;
        values.as_slice()
    }
}

impl<'a, K, V> AsRef<[V]> for Drain<'a, K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_values_slice()
    }
}

impl<'a, K, V> Iterator for Drain<'a, K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next();
        let value = values.next();
        match_kv_same_kind(key, value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, .. } = self;
        keys.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Drain<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next_back();
        let value = values.next_back();
        match_kv_same_kind(key, value)
    }
}

impl<'a, K, V> ExactSizeIterator for Drain<'a, K, V> {}

impl<'a, K, V> FusedIterator for Drain<'a, K, V> {}
