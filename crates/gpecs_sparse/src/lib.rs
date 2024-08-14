//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use alloc::vec;
use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    slice,
};

extern crate alloc;

pub mod arena;
pub mod key;
pub mod prelude;
pub mod set;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SparseItem<E> {
    pub kind: SparseItemKind,
    pub epoch: E,
}

impl<E> SparseItem<E> {
    #[inline]
    pub const fn new(kind: SparseItemKind, epoch: E) -> Self {
        Self { kind, epoch }
    }

    #[inline]
    pub const fn occupied(dense_index: usize, epoch: E) -> Self {
        let kind = SparseItemKind::occupied(dense_index);
        Self::new(kind, epoch)
    }

    #[inline]
    pub const fn vacant(next_vacant: usize, epoch: E) -> Self {
        let kind = SparseItemKind::vacant(next_vacant);
        Self::new(kind, epoch)
    }

    #[inline]
    pub const fn is_occupied(&self) -> bool {
        let Self { kind, .. } = self;
        kind.is_occupied()
    }

    #[inline]
    pub const fn is_vacant(&self) -> bool {
        let Self { kind, .. } = self;
        kind.is_vacant()
    }

    #[inline]
    pub const fn kind(&self) -> &SparseItemKind {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub fn kind_mut(&mut self) -> &mut SparseItemKind {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub const fn dense_index(&self) -> Option<usize> {
        let Self { kind, .. } = self;
        kind.dense_index()
    }

    #[inline]
    pub fn dense_index_mut(&mut self) -> Option<&mut usize> {
        let Self { kind, .. } = self;
        kind.dense_index_mut()
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<usize> {
        let Self { kind, .. } = self;
        kind.next_vacant()
    }

    #[inline]
    pub fn next_vacant_mut(&mut self) -> Option<&mut usize> {
        let Self { kind, .. } = self;
        kind.next_vacant_mut()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum SparseItemKind {
    Occupied { dense_index: usize },
    Vacant { next_vacant: usize },
}

impl SparseItemKind {
    #[inline]
    pub const fn occupied(dense_index: usize) -> Self {
        Self::Occupied { dense_index }
    }

    #[inline]
    pub const fn vacant(next_vacant: usize) -> Self {
        Self::Vacant { next_vacant }
    }

    #[inline]
    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied { .. })
    }

    #[inline]
    pub const fn is_vacant(&self) -> bool {
        matches!(self, Self::Vacant { .. })
    }

    #[inline]
    pub const fn dense_index(&self) -> Option<usize> {
        match self {
            Self::Occupied { dense_index } => Some(*dense_index),
            Self::Vacant { .. } => None,
        }
    }

    #[inline]
    pub fn dense_index_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { dense_index } => Some(dense_index),
            Self::Vacant { .. } => None,
        }
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<usize> {
        match self {
            Self::Occupied { .. } => None,
            Self::Vacant { next_vacant } => Some(*next_vacant),
        }
    }

    #[inline]
    pub fn next_vacant_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { .. } => None,
            Self::Vacant { next_vacant } => Some(next_vacant),
        }
    }
}

fn get_pair_mut<T>(slice: &mut [T], a: usize, b: usize) -> Option<(&mut T, &mut T)> {
    let (first, second) = (usize::min(a, b), usize::max(a, b));

    let [first, .., second] = slice.get_mut(first..=second)? else {
        return None;
    };

    let pair = if a < b {
        (first, second)
    } else {
        (second, first)
    };
    Some(pair)
}

#[cold]
#[track_caller]
#[inline(never)]
const fn check_kv_same_len_failed() -> ! {
    panic!("keys and values should have the same length")
}

#[cold]
#[track_caller]
#[inline(never)]
const fn check_kv_same_capacity_failed() -> ! {
    panic!("keys and values should have the same capacity")
}

#[inline]
#[track_caller]
fn match_kv_same_kind<K, V>(key: Option<K>, value: Option<V>) -> Option<(K, V)> {
    match (key, value) {
        (Some(key), Some(value)) => Some((key, value)),
        (None, None) => None,
        _ => check_kv_same_len_failed(),
    }
}

#[inline]
#[track_caller]
fn unwrap_sparse_item<E>(sparse: &[SparseItem<E>], sparse_index: usize) -> &SparseItem<E> {
    let Some(item) = sparse.get(sparse_index) else {
        check_key_bounds_failed()
    };
    item
}

#[inline]
#[track_caller]
fn unwrap_sparse_item_mut<E>(
    sparse: &mut [SparseItem<E>],
    sparse_index: usize,
) -> &mut SparseItem<E> {
    let Some(item) = sparse.get_mut(sparse_index) else {
        check_key_bounds_failed()
    };
    item
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_dense_index_failed() -> ! {
    panic!("current sparse item should be occupied")
}

#[inline]
#[track_caller]
const fn unwrap_dense_index(kind: &SparseItemKind) -> usize {
    let Some(dense_index) = kind.dense_index() else {
        unwrap_dense_index_failed()
    };
    dense_index
}

#[inline]
#[track_caller]
fn unwrap_dense_index_mut(kind: &mut SparseItemKind) -> &mut usize {
    let Some(dense_index) = kind.dense_index_mut() else {
        unwrap_dense_index_failed()
    };
    dense_index
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_next_vacant_failed() -> ! {
    panic!("current sparse item should be vacant")
}

#[inline]
#[track_caller]
const fn unwrap_next_vacant(kind: &SparseItemKind) -> usize {
    let Some(next_vacant) = kind.next_vacant() else {
        unwrap_next_vacant_failed()
    };
    next_vacant
}

#[inline]
#[track_caller]
fn unwrap_next_vacant_mut(kind: &mut SparseItemKind) -> &mut usize {
    let Some(next_vacant) = kind.next_vacant_mut() else {
        unwrap_next_vacant_failed()
    };
    next_vacant
}

#[inline]
#[track_caller]
fn unwrap_dense_key<K>(keys: &[K], dense_index: usize) -> &K {
    let Some(dense_key) = keys.get(dense_index) else {
        check_dense_index_bounds_failed();
    };
    dense_key
}

#[inline]
#[track_caller]
fn unwrap_dense_key_mut<K>(keys: &mut [K], dense_index: usize) -> &mut K {
    let Some(dense_key) = keys.get_mut(dense_index) else {
        check_dense_index_bounds_failed();
    };
    dense_key
}

#[inline]
#[track_caller]
fn unwrap_dense_value<T>(values: &[T], dense_index: usize) -> &T {
    let Some(dense_value) = values.get(dense_index) else {
        check_dense_index_bounds_failed();
    };
    dense_value
}

#[inline]
#[track_caller]
fn unwrap_dense_value_mut<T>(values: &mut [T], dense_index: usize) -> &mut T {
    let Some(dense_value) = values.get_mut(dense_index) else {
        check_dense_index_bounds_failed();
    };
    dense_value
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_dense_value_pair_mut_failed() -> ! {
    panic!("indices from sparse should be in bounds of dense and differ from each other")
}

#[inline]
#[track_caller]
fn unwrap_dense_value_pair_mut<T>(
    values: &mut [T],
    first_index: usize,
    second_index: usize,
) -> (&mut T, &mut T) {
    let Some(pair) = get_pair_mut(values, first_index, second_index) else {
        unwrap_dense_value_pair_mut_failed()
    };
    pair
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_sparse_items_pair_mut_failed() -> ! {
    panic!("keys should be in bounds of sparse and differ from each other")
}

#[inline]
#[track_caller]
fn unwrap_sparse_items_pair_mut<E>(
    sparse: &mut [SparseItem<E>],
    first_index: usize,
    second_index: usize,
) -> (&mut SparseItem<E>, &mut SparseItem<E>) {
    let Some(pair) = get_pair_mut(sparse, first_index, second_index) else {
        unwrap_sparse_items_pair_mut_failed()
    };
    pair
}

#[inline]
#[track_caller]
fn unwrap_value_from_sparse_index<'a, T, E>(
    sparse_index: usize,
    values: &'a [T],
    sparse: &[SparseItem<E>],
) -> &'a T {
    let sparse_item = unwrap_sparse_item(sparse, sparse_index);
    let dense_index = unwrap_dense_index(&sparse_item.kind);
    unwrap_dense_value(values, dense_index)
}

#[cold]
#[track_caller]
#[inline(never)]
const fn check_dense_index_bounds_failed() -> ! {
    panic!("index from sparse should be in bounds of dense")
}

#[inline]
#[track_caller]
const fn check_dense_index_bounds(dense_index: usize, dense_len: usize) {
    if dense_index < dense_len {
        return;
    }
    check_dense_index_bounds_failed()
}

#[cold]
#[track_caller]
#[inline(never)]
const fn check_key_bounds_failed() -> ! {
    panic!("key from dense should be in bounds of sparse")
}

#[inline]
#[track_caller]
const fn check_key_bounds(key: usize, sparse_len: usize) {
    if key < sparse_len {
        return;
    }
    check_key_bounds_failed()
}

#[cold]
#[track_caller]
#[inline(never)]
const fn check_equal_key_failed() -> ! {
    panic!("provided key and key from dense should be equal")
}

#[inline]
#[track_caller]
fn check_equal_key<K>(key: K, dense_key: K)
where
    K: PartialEq,
{
    if key == dense_key {
        return;
    }
    check_equal_key_failed()
}

#[inline]
#[track_caller]
const fn check_kv_same_len(keys_len: usize, values_len: usize) {
    if keys_len == values_len {
        return;
    }
    check_kv_same_len_failed()
}

#[inline]
#[track_caller]
const fn check_kv_same_capacity(keys_capacity: usize, values_capacity: usize) {
    if keys_capacity == values_capacity {
        return;
    }
    check_kv_same_capacity_failed()
}

#[repr(transparent)]
pub struct Keys<'a, K, V> {
    keys: slice::Iter<'a, K>,
    values: PhantomData<&'a V>,
}

impl<'a, K, V> Keys<'a, K, V> {
    #[inline]
    fn new(keys: slice::Iter<'a, K>) -> Self {
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
    fn new(keys: vec::IntoIter<K>) -> Self {
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
    fn new(values: slice::Iter<'a, V>) -> Self {
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
    fn new(values: slice::IterMut<'a, V>) -> Self {
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
    fn new(values: vec::IntoIter<V>) -> Self {
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
    fn new(keys: slice::Iter<'a, K>, values: slice::Iter<'a, V>) -> Self {
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
    fn new(keys: slice::Iter<'a, K>, values: slice::IterMut<'a, V>) -> Self {
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
    fn new(keys: vec::IntoIter<K>, values: vec::IntoIter<V>) -> Self {
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
    fn new(keys: vec::Drain<'a, K>, values: vec::Drain<'a, V>) -> Self {
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
