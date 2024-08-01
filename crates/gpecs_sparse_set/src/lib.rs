//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::TryReserveError,
    vec::{self, Vec},
};
use core::{
    cmp,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::{replace, swap},
    ops::{Index, IndexMut},
    slice,
};

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

#[inline]
#[track_caller]
fn kv_to_item<K, V>(key: Option<K>, value: Option<V>) -> Option<(K, V)> {
    match (key, value) {
        (Some(key), Some(value)) => Some((key, value)),
        (None, None) => None,
        _ => panic!("keys and values should have the same length"),
    }
}

#[inline]
#[track_caller]
fn get_value<'a, T>(key: usize, values: &'a [T], sparse: &[SparseEntry]) -> &'a T {
    let sparse_entry = sparse
        .get(key)
        .expect("key from dense should be in bounds of sparse");
    let dense_index = sparse_entry
        .dense_index()
        .expect("current sparse entry should be occupied");
    values
        .get(dense_index)
        .expect("index from sparse should be in bounds of dense")
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum SparseEntry {
    Occupied { dense_index: usize },
    Vacant,
}

impl SparseEntry {
    pub const fn occupied(dense_index: usize) -> Self {
        Self::Occupied { dense_index }
    }

    pub const fn vacant() -> Self {
        Self::Vacant
    }

    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied { .. })
    }

    pub const fn is_vacant(&self) -> bool {
        matches!(self, Self::Vacant)
    }

    pub const fn dense_index(&self) -> Option<usize> {
        match self {
            Self::Occupied { dense_index } => Some(*dense_index),
            Self::Vacant => None,
        }
    }

    pub fn dense_index_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { dense_index } => Some(dense_index),
            Self::Vacant => None,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SparseSet<T> {
    dense_keys: Vec<usize>,
    dense_values: Vec<T>,
    sparse: Vec<SparseEntry>,
}

impl<T> SparseSet<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            dense_keys: Vec::new(),
            dense_values: Vec::new(),
            sparse: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(dense: usize, sparse: usize) -> Self {
        Self {
            dense_keys: Vec::with_capacity(dense),
            dense_values: Vec::with_capacity(dense),
            sparse: Vec::with_capacity(sparse),
        }
    }

    #[inline]
    pub fn try_with_capacity(dense: usize, sparse: usize) -> Result<Self, TryReserveError> {
        let mut me = Self::new();
        me.try_reserve(dense, sparse)?;
        Ok(me)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        debug_assert_eq!(dense_keys.len(), dense_values.len());
        dense_keys.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn sparse_len(&self) -> usize {
        let Self { sparse, .. } = self;
        sparse.len()
    }

    #[inline]
    pub fn sparse_is_empty(&self) -> bool {
        self.sparse_len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        debug_assert_eq!(dense_keys.capacity(), dense_values.capacity());
        dense_keys.capacity()
    }

    #[inline]
    pub fn sparse_capacity(&self) -> usize {
        let Self { sparse, .. } = self;
        sparse.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.reserve(additional_dense);
        dense_values.reserve(additional_dense);
        sparse.reserve(additional_sparse);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.reserve_exact(additional_dense);
        dense_values.reserve_exact(additional_dense);
        sparse.reserve_exact(additional_sparse);
    }

    #[inline]
    pub fn try_reserve(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.try_reserve(additional_dense)?;
        dense_values.try_reserve(additional_dense)?;
        sparse.try_reserve(additional_sparse)?;
        Ok(())
    }

    #[inline]
    pub fn try_reserve_exact(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.try_reserve_exact(additional_dense)?;
        dense_values.try_reserve_exact(additional_dense)?;
        sparse.try_reserve_exact(additional_sparse)?;
        Ok(())
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.shrink_to_fit();
        dense_values.shrink_to_fit();
        sparse.shrink_to_fit();
    }

    #[inline]
    pub fn dense_shrink_to_fit(&mut self) {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        dense_keys.shrink_to_fit();
        dense_values.shrink_to_fit();
    }

    #[inline]
    pub fn sparse_shrink_to_fit(&mut self) {
        let Self { sparse, .. } = self;
        sparse.shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.shrink_to(min_capacity);
        dense_values.shrink_to(min_capacity);
        sparse.shrink_to(min_capacity);
    }

    #[inline]
    pub fn dense_shrink_to(&mut self, min_capacity: usize) {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        dense_keys.shrink_to(min_capacity);
        dense_values.shrink_to(min_capacity);
    }

    #[inline]
    pub fn sparse_shrink_to(&mut self, min_capacity: usize) {
        let Self { sparse, .. } = self;
        sparse.shrink_to(min_capacity);
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        let Self { dense_values, .. } = self;
        dense_values.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let Self { dense_values, .. } = self;
        dense_values.as_mut_slice()
    }

    #[inline]
    pub fn into_boxed_slice(self) -> Box<[T]> {
        let Self { dense_values, .. } = self;
        dense_values.into_boxed_slice()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        let Self { dense_values, .. } = self;
        dense_values.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        let Self { dense_values, .. } = self;
        dense_values.as_mut_ptr()
    }

    pub fn insert(&mut self, key: usize, value: T) -> Option<T> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        if key >= sparse.len() {
            sparse.resize(key.saturating_add(1), SparseEntry::Vacant);
        }

        let sparse = sparse.as_mut_slice();
        if let SparseEntry::Occupied { dense_index } = sparse[key] {
            let entry_value = dense_values
                .get_mut(dense_index)
                .expect("index from sparse should be in bounds of dense");
            let value = replace(entry_value, value);
            return Some(value);
        }

        debug_assert_eq!(dense_keys.len(), dense_values.len());
        dense_keys.push(key);
        dense_values.push(value);
        sparse[key] = SparseEntry::occupied(dense_keys.len() - 1);

        None
    }

    pub fn try_insert(&mut self, key: usize, value: T) -> Result<Option<T>, TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        if key >= sparse.len() {
            let new_sparse_len = key.saturating_add(1);
            sparse.try_reserve(new_sparse_len - sparse.len())?;
            sparse.resize(new_sparse_len, SparseEntry::Vacant);
        }

        let sparse = sparse.as_mut_slice();
        if let SparseEntry::Occupied { dense_index } = sparse[key] {
            let entry_value = dense_values
                .get_mut(dense_index)
                .expect("index from sparse should be in bounds of dense");
            let value = replace(entry_value, value);
            return Ok(Some(value));
        }

        debug_assert_eq!(dense_keys.len(), dense_values.len());
        dense_keys.try_reserve(1)?;
        dense_values.try_reserve(1)?;

        dense_keys.push(key);
        dense_values.push(value);
        sparse[key] = SparseEntry::occupied(dense_keys.len() - 1);

        Ok(None)
    }

    pub fn push(&mut self, value: T) -> usize {
        let Self { sparse, .. } = self;

        let key = sparse
            .iter()
            .position(SparseEntry::is_vacant)
            .unwrap_or(self.sparse.len());
        self.insert(key, value);

        key
    }

    pub fn try_push(&mut self, value: T) -> Result<usize, TryReserveError> {
        let Self { sparse, .. } = self;

        let key = sparse
            .iter()
            .position(SparseEntry::is_vacant)
            .unwrap_or(self.sparse.len());
        self.try_insert(key, value)?;

        Ok(key)
    }

    pub fn swap(&mut self, first_key: usize, second_key: usize) {
        let Self {
            dense_values,
            sparse,
            ..
        } = self;

        if first_key == second_key {
            return;
        }

        let Some(first_index) = sparse.get(first_key).and_then(SparseEntry::dense_index) else {
            return;
        };
        let Some(second_index) = sparse.get(second_key).and_then(SparseEntry::dense_index) else {
            return;
        };

        let (first_value, second_value) = get_pair_mut(dense_values, first_index, second_index)
            .expect("indices from sparse should be in bounds of dense and differ from each other");
        swap(first_value, second_value);
    }

    pub fn swap_remove(&mut self, key: usize) -> Option<T> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let dense_index = sparse.get(key).and_then(SparseEntry::dense_index)?;
        assert!(
            dense_index < dense_keys.len(),
            "index from sparse should be in bounds of dense",
        );

        debug_assert_eq!(dense_keys.len(), dense_values.len());
        let value = dense_values.swap_remove(dense_index);
        let dense_key = dense_keys.swap_remove(dense_index);
        debug_assert_eq!(key, dense_key);

        sparse[dense_keys.len()] = sparse[key];
        sparse[key] = SparseEntry::Vacant;

        Some(value)
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let dense_index = sparse.get(key).and_then(SparseEntry::dense_index)?;
        assert!(
            dense_index < dense_keys.len(),
            "index from sparse should be in bounds of dense",
        );

        debug_assert_eq!(dense_keys.len(), dense_values.len());
        let value = dense_values.remove(dense_index);
        let dense_key = dense_keys.remove(dense_index);
        debug_assert_eq!(key, dense_key);

        for key in dense_keys.iter().copied().skip(dense_index) {
            let sparse_entry = sparse
                .get_mut(key)
                .expect("key from dense should be in bounds of sparse");
            let dense_index = sparse_entry
                .dense_index_mut()
                .expect("current sparse entry should be occupied");
            *dense_index -= 1;
        }
        sparse[key] = SparseEntry::Vacant;

        Some(value)
    }

    pub fn pop(&mut self) -> Option<(usize, T)> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let key = dense_keys.pop();
        let value = dense_values.pop();
        let (key, value) = kv_to_item(key, value)?;

        assert!(
            key < sparse.len(),
            "key from dense should be in bounds of sparse",
        );
        sparse[key] = SparseEntry::Vacant;

        Some((key, value))
    }

    pub fn truncate(&mut self, dense_len: usize, sparse_len: usize) {
        for dense_index in (dense_len..self.len()).rev() {
            let key = self.dense_keys[dense_index];
            self.remove(key);
        }
        self.dense_keys.truncate(dense_len);
        self.dense_values.truncate(dense_len);

        for key in sparse_len..self.sparse_len() {
            self.remove(key);
        }
        self.sparse.truncate(sparse_len);
    }

    pub fn drain(&mut self) -> Drain<'_, T> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let keys = dense_keys.drain(..);
        let values = dense_values.drain(..);
        debug_assert_eq!(keys.len(), values.len());
        sparse.clear();

        Drain { keys, values }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, &mut T) -> bool,
    {
        for dense_index in (0..self.len()).rev() {
            let key = self.dense_keys[dense_index];
            let value = self.dense_values.index_mut(dense_index);
            if !f(key, value) {
                self.remove(key);
            }
        }
    }

    #[inline]
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| get_value(key, values, sparse))
        });
    }

    #[inline]
    pub fn sort_keys(&mut self) {
        self.sort_impl(|keys, _, _| keys.sort());
    }

    #[inline]
    pub fn sort_by<F>(&mut self, mut f: F)
    where
        F: FnMut((usize, &T), (usize, &T)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by(|&lhs_key, &rhs_key| {
                let lhs_value = get_value(lhs_key, values, sparse);
                let rhs_value = get_value(rhs_key, values, sparse);
                let lhs = (lhs_key, lhs_value);
                let rhs = (rhs_key, rhs_value);
                f(lhs, rhs)
            })
        });
    }

    #[inline]
    pub fn sort_by_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut((usize, &T)) -> K,
        K: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_key(|&key| {
                let value = get_value(key, values, sparse);
                f((key, value))
            })
        });
    }

    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut((usize, &T)) -> K,
        K: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let value = get_value(key, values, sparse);
                f((key, value))
            })
        });
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| get_value(key, values, sparse))
        });
    }

    #[inline]
    pub fn sort_keys_unstable(&mut self) {
        self.sort_impl(|keys, _, _| keys.sort_unstable());
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, mut f: F)
    where
        F: FnMut((usize, &T), (usize, &T)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by(|&lhs_key, &rhs_key| {
                let lhs_value = get_value(lhs_key, values, sparse);
                let rhs_value = get_value(rhs_key, values, sparse);
                let lhs = (lhs_key, lhs_value);
                let rhs = (rhs_key, rhs_value);
                f(lhs, rhs)
            })
        });
    }

    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, mut f: F)
    where
        F: FnMut((usize, &T)) -> K,
        K: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let value = get_value(key, values, sparse);
                f((key, value))
            })
        });
    }

    // https://github.com/skypjack/entt/blob/8b0ef2b94234def2053c9a8a2591f4a5e87cf0ea/src/entt/entity/sparse_set.hpp#L964
    fn sort_impl<SortKeys>(&mut self, sort_keys: SortKeys)
    where
        SortKeys: FnOnce(&mut [usize], &[T], &[SparseEntry]),
    {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let dense_keys = dense_keys.as_mut_slice();
        let dense_values = dense_values.as_mut_slice();
        let sparse = sparse.as_mut_slice();
        debug_assert_eq!(dense_keys.len(), dense_values.len());

        sort_keys(dense_keys, dense_values, sparse);

        for pos in 0..dense_keys.len() {
            let mut curr = pos;
            let mut next = sparse[dense_keys[curr]].dense_index().unwrap();

            while curr != next {
                let (curr_entry, next_entry) =
                    get_pair_mut(sparse, dense_keys[curr], dense_keys[next]).unwrap();
                let curr_dense_index = curr_entry.dense_index_mut().unwrap();
                let next_dense_index = next_entry.dense_index_mut().unwrap();

                dense_values.swap(*curr_dense_index, *next_dense_index);

                *curr_dense_index = curr;
                curr = next;
                next = *next_dense_index;
            }
        }
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_entry = sparse.get(key).copied()?;
        let dense_index = sparse_entry.dense_index()?;

        let value = dense_values
            .get(dense_index)
            .expect("index from sparse should be in bounds of dense");
        let dense_key = dense_keys
            .get(dense_index)
            .copied()
            .expect("index from sparse should be in bounds of dense");
        debug_assert_eq!(key, dense_key);

        Some(value)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_entry = sparse.get(key).copied()?;
        let dense_index = sparse_entry.dense_index()?;

        let value = dense_values
            .get_mut(dense_index)
            .expect("index from sparse should be in bounds of dense");
        let dense_key = dense_keys
            .get(dense_index)
            .copied()
            .expect("index from sparse should be in bounds of dense");
        debug_assert_eq!(key, dense_key);

        Some(value)
    }

    pub fn contains_key(&self, key: usize) -> bool {
        let Self {
            dense_keys, sparse, ..
        } = self;

        let Some(sparse_entry) = sparse.get(key).copied() else {
            return false;
        };
        let SparseEntry::Occupied { dense_index } = sparse_entry else {
            return false;
        };

        debug_assert!(
            dense_index < dense_keys.len(),
            "index from sparse should be in bounds of dense",
        );
        true
    }

    #[inline]
    pub fn clear(&mut self) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.clear();
        dense_values.clear();
        sparse.clear();
    }

    // TODO operations from `Vec<T>` and `HashMap<K, V>` if possible
    // TODO Entry API

    #[inline]
    pub fn keys(&self) -> Keys<'_, T> {
        let Self { dense_keys, .. } = self;

        let keys = dense_keys.iter();
        let phantom = PhantomData;
        Keys { keys, phantom }
    }

    #[inline]
    pub fn into_keys(self) -> IntoKeys<T> {
        let Self { dense_keys, .. } = self;

        let keys = dense_keys.into_iter();
        let phantom = PhantomData;
        IntoKeys { keys, phantom }
    }

    #[inline]
    pub fn values(&self) -> Values<'_, T> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter();
        Values { values }
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, T> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter_mut();
        ValuesMut { values }
    }

    #[inline]
    pub fn into_values(self) -> IntoValues<T> {
        let Self { dense_values, .. } = self;

        let values = dense_values.into_iter();
        IntoValues { values }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.iter();
        let values = dense_values.iter();
        debug_assert_eq!(keys.len(), values.len());

        Iter { keys, values }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.iter();
        let values = dense_values.iter_mut();
        debug_assert_eq!(keys.len(), values.len());

        IterMut { keys, values }
    }
}

impl<T> Index<usize> for SparseSet<T> {
    type Output = T;

    #[inline]
    fn index(&self, key: usize) -> &Self::Output {
        match self.get(key) {
            Some(value) => value,
            None => panic!("key {key} not found"),
        }
    }
}

impl<T> IndexMut<usize> for SparseSet<T> {
    #[inline]
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        match self.get_mut(key) {
            Some(value) => value,
            None => panic!("key {key} not found"),
        }
    }
}

impl<T> AsRef<[T]> for SparseSet<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for SparseSet<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> AsRef<SparseSet<T>> for SparseSet<T> {
    #[inline]
    fn as_ref(&self) -> &SparseSet<T> {
        self
    }
}

impl<T> AsMut<SparseSet<T>> for SparseSet<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut SparseSet<T> {
        self
    }
}

impl<'a, T> IntoIterator for &'a SparseSet<T> {
    type Item = (&'a usize, &'a T);

    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SparseSet<T> {
    type Item = (&'a usize, &'a mut T);

    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for SparseSet<T> {
    type Item = (usize, T);

    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.into_iter();
        let values = dense_values.into_iter();
        debug_assert_eq!(keys.len(), values.len());

        IntoIter { keys, values }
    }
}

impl<T> FromIterator<(usize, T)> for SparseSet<T> {
    fn from_iter<I: IntoIterator<Item = (usize, T)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let iter_len = {
            let (lower, upper) = iter.size_hint();
            upper.unwrap_or(lower)
        };

        let mut me = Self::with_capacity(iter_len, iter_len);
        for (key, value) in iter {
            me.insert(key, value);
        }

        me
    }
}

impl<T> FromIterator<T> for SparseSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let dense_values: Vec<_> = iter.into_iter().collect();

        let len = dense_values.len();
        let dense_keys = (0..len).collect();
        let sparse = (0..len).map(SparseEntry::occupied).collect();

        Self {
            dense_keys,
            dense_values,
            sparse,
        }
    }
}

impl<T> Extend<(usize, T)> for SparseSet<T> {
    fn extend<I: IntoIterator<Item = (usize, T)>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let iter_len = {
            let (lower, upper) = iter.size_hint();
            upper.unwrap_or(lower)
        };
        self.reserve(iter_len, iter_len);

        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<T> Extend<T> for SparseSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let iter_len = {
            let (lower, upper) = iter.size_hint();
            upper.unwrap_or(lower)
        };
        self.reserve(iter_len, iter_len);

        let mut maybe_vacant_keys = 0..self.sparse.len();
        for value in iter {
            let key = maybe_vacant_keys
                .find(|&key| self.sparse[key].is_vacant())
                .unwrap_or(self.sparse.len());
            self.insert(key, value);
        }
    }
}

pub struct Keys<'a, T> {
    keys: slice::Iter<'a, usize>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> Keys<'a, T> {
    #[inline]
    pub fn as_slice(&self) -> &'a [usize] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }
}

impl<'a, T> Debug for Keys<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("Keys").field(keys).finish()
    }
}

impl<'a, T> Default for Keys<'a, T> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let phantom = Default::default();
        Self { keys, phantom }
    }
}

impl<'a, T> Clone for Keys<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, phantom } = self;

        let keys = keys.clone();
        let phantom = *phantom;
        Self { keys, phantom }
    }
}

impl<'a, T> AsRef<[usize]> for Keys<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[usize] {
        self.as_slice()
    }
}

impl<'a, T> Iterator for Keys<'a, T> {
    type Item = &'a usize;

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

impl<'a, T> DoubleEndedIterator for Keys<'a, T> {
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

impl<'a, T> ExactSizeIterator for Keys<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        let Self { keys, .. } = self;
        keys.len()
    }
}

impl<'a, T> FusedIterator for Keys<'a, T> {}

pub struct IntoKeys<T> {
    keys: vec::IntoIter<usize>,
    phantom: PhantomData<T>,
}

impl<T> IntoKeys<T> {
    #[inline]
    pub fn as_slice(&self) -> &[usize] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [usize] {
        let Self { keys, .. } = self;
        keys.as_mut_slice()
    }
}

impl<T> Debug for IntoKeys<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("IntoKeys").field(keys).finish()
    }
}

impl<T> Default for IntoKeys<T> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let phantom = Default::default();
        Self { keys, phantom }
    }
}

impl<T> Clone for IntoKeys<T> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, phantom } = self;

        let keys = keys.clone();
        let phantom = *phantom;
        Self { keys, phantom }
    }
}

impl<T> AsRef<[usize]> for IntoKeys<T> {
    #[inline]
    fn as_ref(&self) -> &[usize] {
        self.as_slice()
    }
}

impl<T> AsMut<[usize]> for IntoKeys<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [usize] {
        self.as_mut_slice()
    }
}

impl<T> Iterator for IntoKeys<T> {
    type Item = usize;

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

impl<T> DoubleEndedIterator for IntoKeys<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.next_back()
    }
}

impl<T> ExactSizeIterator for IntoKeys<T> {}

impl<T> FusedIterator for IntoKeys<T> {}

pub struct Values<'a, T> {
    values: slice::Iter<'a, T>,
}

impl<'a, T> Values<'a, T> {
    #[inline]
    pub fn as_slice(&self) -> &'a [T] {
        let Self { values } = self;
        values.as_slice()
    }
}

impl<'a, T> Debug for Values<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("Values").field(values).finish()
    }
}

impl<'a, T> Default for Values<'a, T> {
    #[inline]
    fn default() -> Self {
        let values = Default::default();
        Self { values }
    }
}

impl<'a, T> Clone for Values<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { values } = self;

        let values = values.clone();
        Self { values }
    }
}

impl<'a, T> AsRef<[T]> for Values<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { values } = self;
        values.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth(n)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { values } = self;
        values.for_each(f)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { values } = self;
        values.fold(init, f)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { values } = self;
        values.find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { values } = self;
        values.find_map(f)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.position(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.rposition(predicate)
    }
}

impl<'a, T> DoubleEndedIterator for Values<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth_back(n)
    }
}

impl<'a, T> ExactSizeIterator for Values<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        let Self { values } = self;
        values.len()
    }
}

impl<'a, T> FusedIterator for Values<'a, T> {}

pub struct ValuesMut<'a, T> {
    values: slice::IterMut<'a, T>,
}

impl<'a, T> ValuesMut<'a, T> {
    #[inline]
    pub fn into_slice(self) -> &'a [T] {
        let Self { values } = self;
        values.into_slice()
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        let Self { values } = self;
        values.as_slice()
    }
}

impl<'a, T> Debug for ValuesMut<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("ValuesMut").field(values).finish()
    }
}

impl<'a, T> Default for ValuesMut<'a, T> {
    #[inline]
    fn default() -> Self {
        let values = Default::default();
        Self { values }
    }
}

impl<'a, T> AsRef<[T]> for ValuesMut<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { values } = self;
        values.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth(n)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { values } = self;
        values.for_each(f)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { values } = self;
        values.fold(init, f)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { values } = self;
        values.find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { values } = self;
        values.find_map(f)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.position(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.rposition(predicate)
    }
}

impl<'a, T> DoubleEndedIterator for ValuesMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth_back(n)
    }
}

impl<'a, T> ExactSizeIterator for ValuesMut<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        let Self { values } = self;
        values.len()
    }
}

impl<'a, T> FusedIterator for ValuesMut<'a, T> {}

#[derive(Clone)]
pub struct IntoValues<T> {
    values: vec::IntoIter<T>,
}

impl<T> IntoValues<T> {
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        let Self { values } = self;
        values.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let Self { values } = self;
        values.as_mut_slice()
    }
}

impl<T> Debug for IntoValues<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slice();
        f.debug_tuple("IntoValues").field(values).finish()
    }
}

impl<T> Default for IntoValues<T> {
    #[inline]
    fn default() -> Self {
        let values = Default::default();
        Self { values }
    }
}

impl<T> AsRef<[T]> for IntoValues<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for IntoValues<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> Iterator for IntoValues<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { values } = self;
        values.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { values } = self;
        values.fold(init, f)
    }
}

impl<T> DoubleEndedIterator for IntoValues<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next_back()
    }
}

impl<T> ExactSizeIterator for IntoValues<T> {}

impl<T> FusedIterator for IntoValues<T> {}

pub struct Iter<'a, T> {
    keys: slice::Iter<'a, usize>,
    values: slice::Iter<'a, T>,
}

impl<'a, T> Iter<'a, T> {
    #[inline]
    pub fn as_keys_slice(&self) -> &'a [usize] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_values_slice(&self) -> &'a [T] {
        let Self { values, .. } = self;
        values.as_slice()
    }

    #[inline]
    pub fn as_slices(&self) -> (&'a [usize], &'a [T]) {
        let Self { keys, values } = self;
        (keys.as_slice(), values.as_slice())
    }
}

impl<'a, T> Debug for Iter<'a, T>
where
    T: Debug,
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

impl<'a, T> Default for Iter<'a, T> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Default::default();
        Self { keys, values }
    }
}

impl<'a, T> Clone for Iter<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, values } = self;

        let keys = keys.clone();
        let values = values.clone();
        Self { keys, values }
    }
}

impl<'a, T> AsRef<[T]> for Iter<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (&'a usize, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next();
        let value = values.next();
        kv_to_item(key, value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, values } = self;

        debug_assert_eq!(keys.size_hint(), values.size_hint());
        keys.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, values } = self;

        debug_assert_eq!(keys.len(), values.len());
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
        kv_to_item(key, value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.nth(n);
        let value = values.nth(n);
        kv_to_item(key, value)
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

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next_back();
        let value = values.next_back();
        kv_to_item(key, value)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.nth_back(n);
        let value = values.nth_back(n);
        kv_to_item(key, value)
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        let Self { keys, values } = self;

        debug_assert_eq!(keys.len(), values.len());
        keys.len()
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> {}

pub struct IterMut<'a, T> {
    keys: slice::Iter<'a, usize>,
    values: slice::IterMut<'a, T>,
}

impl<'a, T> IterMut<'a, T> {
    #[inline]
    pub fn into_keys_slice(self) -> &'a [usize] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &'a [usize] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn into_values_slice(self) -> &'a mut [T] {
        let Self { values, .. } = self;
        values.into_slice()
    }

    #[inline]
    pub fn as_values_slice(&self) -> &[T] {
        let Self { values, .. } = self;
        values.as_slice()
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [usize], &'a mut [T]) {
        let Self { keys, values } = self;
        (keys.as_slice(), values.into_slice())
    }

    #[inline]
    pub fn as_slices(&self) -> (&'a [usize], &[T]) {
        let Self { keys, values } = self;
        (keys.as_slice(), values.as_slice())
    }
}

impl<'a, T> Debug for IterMut<'a, T>
where
    T: Debug,
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

impl<'a, T> Default for IterMut<'a, T> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Default::default();
        Self { keys, values }
    }
}

impl<'a, T> AsRef<[T]> for IterMut<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (&'a usize, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next();
        let value = values.next();
        kv_to_item(key, value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, values } = self;

        debug_assert_eq!(keys.size_hint(), values.size_hint());
        keys.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, values } = self;

        debug_assert_eq!(keys.len(), values.len());
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
        kv_to_item(key, value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.nth(n);
        let value = values.nth(n);
        kv_to_item(key, value)
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

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next_back();
        let value = values.next_back();
        kv_to_item(key, value)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.nth_back(n);
        let value = values.nth_back(n);
        kv_to_item(key, value)
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        let Self { keys, values } = self;

        debug_assert_eq!(keys.len(), values.len());
        keys.len()
    }
}

impl<'a, T> FusedIterator for IterMut<'a, T> {}

#[derive(Clone)]
pub struct IntoIter<T> {
    keys: vec::IntoIter<usize>,
    values: vec::IntoIter<T>,
}

impl<T> IntoIter<T> {
    #[inline]
    pub fn as_keys_slice(&self) -> &[usize] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_keys_mut_slice(&mut self) -> &mut [usize] {
        let Self { keys, .. } = self;
        keys.as_mut_slice()
    }

    #[inline]
    pub fn as_values_slice(&self) -> &[T] {
        let Self { values, .. } = self;
        values.as_slice()
    }

    #[inline]
    pub fn as_values_mut_slice(&mut self) -> &mut [T] {
        let Self { values, .. } = self;
        values.as_mut_slice()
    }

    #[inline]
    pub fn as_slices(&self) -> (&[usize], &[T]) {
        let Self { keys, values } = self;
        (keys.as_slice(), values.as_slice())
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [usize], &mut [T]) {
        let Self { keys, values } = self;
        (keys.as_mut_slice(), values.as_mut_slice())
    }
}

impl<T> Debug for IntoIter<T>
where
    T: Debug,
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

impl<T> Default for IntoIter<T> {
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Default::default();
        Self { keys, values }
    }
}

impl<T> AsRef<[T]> for IntoIter<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<T> AsMut<[T]> for IntoIter<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_values_mut_slice()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = (usize, T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next();
        let value = values.next();
        kv_to_item(key, value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, values } = self;

        debug_assert_eq!(keys.size_hint(), values.size_hint());
        keys.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, values } = self;

        debug_assert_eq!(keys.len(), values.len());
        keys.count()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next_back();
        let value = values.next_back();
        kv_to_item(key, value)
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> FusedIterator for IntoIter<T> {}

pub struct Drain<'a, T> {
    keys: vec::Drain<'a, usize>,
    values: vec::Drain<'a, T>,
}

impl<'a, T: Debug> Debug for Drain<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_keys_slice();
        let values = &self.as_values_slice();
        f.debug_struct("Drain")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'a, T> Drain<'a, T> {
    #[inline]
    pub fn as_keys_slice(&self) -> &[usize] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

    #[inline]
    pub fn as_values_slice(&self) -> &[T] {
        let Self { values, .. } = self;
        values.as_slice()
    }
}

impl<'a, T> AsRef<[T]> for Drain<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_values_slice()
    }
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = (usize, T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next();
        let value = values.next();
        kv_to_item(key, value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, values } = self;

        debug_assert_eq!(keys.size_hint(), values.size_hint());
        keys.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, values } = self;

        let key = keys.next_back();
        let value = values.next_back();
        kv_to_item(key, value)
    }
}

impl<'a, T> ExactSizeIterator for Drain<'a, T> {}

impl<'a, T> FusedIterator for Drain<'a, T> {}

#[cfg(test)]
mod tests {
    use std::{mem::forget, ops::Not};

    use crate::SparseSet;

    #[test]
    fn empty() {
        let sparse_set = SparseSet::<i32>::new();
        assert!(sparse_set.is_empty());
    }

    #[test]
    fn with_capacity() {
        let sparse_set = SparseSet::<i32>::with_capacity(10, 10);
        assert!(sparse_set.is_empty());
        assert!(sparse_set.capacity() >= 10);
        assert!(sparse_set.sparse_capacity() >= 10);
    }

    #[test]
    fn empty_keys() {
        let sparse_set = SparseSet::<i32>::new();

        let keys = sparse_set.keys();
        assert_eq!(keys.len(), 0);
        assert_eq!(keys.as_slice(), &[]);
    }

    #[test]
    fn empty_into_keys() {
        let sparse_set = SparseSet::<i32>::new();

        let keys = sparse_set.into_keys();
        assert_eq!(keys.len(), 0);
        assert_eq!(keys.as_slice(), &[]);
    }

    #[test]
    fn empty_values() {
        let sparse_set = SparseSet::<i32>::new();

        let values = sparse_set.values();
        assert_eq!(values.len(), 0);
        assert_eq!(values.as_slice(), &[]);
    }

    #[test]
    fn empty_values_mut() {
        let mut sparse_set = SparseSet::<i32>::new();
        let values_mut = sparse_set.values_mut();

        assert_eq!(values_mut.len(), 0);
        assert_eq!(values_mut.into_slice(), &mut []);
    }

    #[test]
    fn empty_into_values() {
        let sparse_set = SparseSet::<i32>::new();

        let values = sparse_set.into_values();
        assert_eq!(values.len(), 0);
        assert_eq!(values.as_slice(), &[]);
    }

    #[test]
    fn empty_iter() {
        let sparse_set = SparseSet::<i32>::new();

        let iter = sparse_set.iter();
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.as_keys_slice(), &[]);
        assert_eq!(iter.as_values_slice(), &[]);
    }

    #[test]
    fn empty_iter_mut() {
        let mut sparse_set = SparseSet::<i32>::new();
        let iter_mut = sparse_set.iter_mut();

        assert_eq!(iter_mut.len(), 0);
        assert_eq!(iter_mut.as_keys_slice(), &[]);
        assert_eq!(iter_mut.into_values_slice(), &mut []);
    }

    #[test]
    fn empty_into_iter() {
        let sparse_set = SparseSet::<i32>::new();
        let into_iter = sparse_set.into_iter();

        assert_eq!(into_iter.len(), 0);
        assert_eq!(into_iter.as_keys_slice(), &[]);
        assert_eq!(into_iter.as_values_slice(), &[]);
    }

    #[test]
    fn empty_insert_one() {
        let mut sparse_set = SparseSet::new();
        let previous = sparse_set.insert(0, 42);
        assert_eq!(previous, None);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn with_capacity_insert_one() {
        let mut sparse_set = SparseSet::with_capacity(10, 10);
        let previous = sparse_set.insert(0, 42);
        assert_eq!(previous, None);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn empty_insert_one_mutate() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set[0] = 43;

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&43));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn with_capacity_insert_one_mutate() {
        let mut sparse_set = SparseSet::with_capacity(10, 10);
        sparse_set.insert(0, 42);
        sparse_set[0] = 43;

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&43));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn empty_insert_far() {
        let mut sparse_set = SparseSet::new();

        let (key, value) = (3, 42);
        sparse_set.insert(key, value);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let (key, value) = (6, 69);
        sparse_set.insert(key, value);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn empty_insert_far_remove() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(3, 42);
        sparse_set.insert(1, 69);

        let key = 3;
        let value = sparse_set.remove(key).unwrap();

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        let key = 1;
        let value = sparse_set.remove(key).unwrap();

        assert_eq!(value, 69);
        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn empty_push() {
        let mut sparse_set = SparseSet::new();

        let key = sparse_set.push(42);
        assert_eq!(key, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&42));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn empty_pop() {
        let mut sparse_set = SparseSet::<i32>::new();

        let popped = sparse_set.pop();
        assert_eq!(popped, None);
        assert_eq!(sparse_set.len(), 0);
    }

    #[test]
    fn one_item_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(0), None);
        assert!(sparse_set.contains_key(0).not());
    }

    #[test]
    fn one_item_swap_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(0), None);
        assert!(sparse_set.contains_key(0).not());
    }

    #[test]
    fn one_item_swap() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        sparse_set.swap(0, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains_key(0));

        sparse_set.swap(0, 1);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn one_item_keys() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(0, 42);

        let keys = sparse_set.keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.as_slice(), &[0]);
    }

    #[test]
    fn one_item_into_keys() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(0, 42);

        let keys = sparse_set.into_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.as_slice(), &[0]);
    }

    #[test]
    fn one_item_values() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(0, 42);

        let values = sparse_set.values();
        assert_eq!(values.len(), 1);
        assert_eq!(values.as_slice(), &[42]);
    }

    #[test]
    fn one_item_values_mut() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(0, 42);

        let values_mut = sparse_set.values_mut();
        assert_eq!(values_mut.len(), 1);
        assert_eq!(values_mut.into_slice(), &mut [42]);
    }

    #[test]
    fn one_item_into_values() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(0, 42);

        let values = sparse_set.into_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values.as_slice(), &[42]);
    }

    #[test]
    fn one_item_iter() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(0, 42);

        let iter = sparse_set.iter();
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.as_keys_slice(), &[0]);
        assert_eq!(iter.as_values_slice(), &[42]);
    }

    #[test]
    fn one_item_iter_mut() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(0, 42);

        let iter_mut = sparse_set.iter_mut();
        assert_eq!(iter_mut.len(), 1);
        assert_eq!(iter_mut.as_keys_slice(), &[0]);
        assert_eq!(iter_mut.into_values_slice(), &mut [42]);
    }

    #[test]
    fn one_item_into_iter() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(0, 42);

        let into_iter = sparse_set.into_iter();
        assert_eq!(into_iter.len(), 1);
        assert_eq!(into_iter.as_keys_slice(), &[0]);
        assert_eq!(into_iter.as_values_slice(), &[42]);
    }

    #[test]
    fn two_items_insert_first() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let previous = sparse_set.insert(0, 34);
        assert_eq!(previous, Some(42));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), Some(&69));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_insert_second() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let previous = sparse_set.insert(1, 34);
        assert_eq!(previous, Some(69));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&34));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_remove_first() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), None);
        assert_eq!(sparse_set.get(1), Some(&69));
        assert!(sparse_set.contains_key(0).not());
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_swap_remove_first() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), None);
        assert_eq!(sparse_set.get(1), Some(&69));
        assert!(sparse_set.contains_key(0).not());
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_remove_second() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let removed = sparse_set.remove(1);
        assert_eq!(removed, Some(69));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), None);
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1).not());
    }

    #[test]
    fn two_items_swap_remove_second() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let removed = sparse_set.swap_remove(1);
        assert_eq!(removed, Some(69));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), None);
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1).not());
    }

    #[test]
    fn two_items_remove_one_insert_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some(42));
        assert_eq!(sparse_set.get(0), None);

        sparse_set.insert(0, 34);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), Some(&69));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_swap_remove_one_insert_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some(42));
        assert_eq!(sparse_set.get(0), None);

        sparse_set.insert(0, 34);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), Some(&69));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_remove_one_push_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some(42));
        assert_eq!(sparse_set.get(0), None);

        let key = sparse_set.push(34);
        assert_eq!(key, 0);

        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), Some(&69));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_swap_remove_one_push_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some(42));
        assert_eq!(sparse_set.get(0), None);

        let key = sparse_set.push(34);
        assert_eq!(key, 0);

        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), Some(&69));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_swap() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        sparse_set.swap(0, 0);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        sparse_set.swap(0, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&42));

        sparse_set.swap(1, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&42));
    }

    #[test]
    fn two_items_pop() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(5, 42);
        sparse_set.insert(2, 69);

        let popped = sparse_set.pop();
        assert_eq!(popped, Some((2, 69)));
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(5), Some(&42));
        assert_eq!(sparse_set.get(2), None);
    }

    #[test]
    fn three_items_remove_middle() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(2, 69);

        let removed = sparse_set.remove(1);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), None);
        assert_eq!(sparse_set.get(2), Some(&69));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1).not());
        assert!(sparse_set.contains_key(2));
    }

    #[test]
    fn three_items_swap_remove_middle() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(2, 69);

        let removed = sparse_set.swap_remove(1);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), None);
        assert_eq!(sparse_set.get(2), Some(&69));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1).not());
        assert!(sparse_set.contains_key(2));
    }

    #[test]
    fn three_items_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let keys = sparse_set.keys();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys.as_slice(), &[2, 1, 5]);
    }

    #[test]
    fn three_items_into_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let keys = sparse_set.into_keys();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys.as_slice(), &[2, 1, 5]);
    }

    #[test]
    fn three_items_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let values = sparse_set.values();
        assert_eq!(values.len(), 3);
        assert_eq!(values.as_slice(), &[34, 42, 69]);
    }

    #[test]
    fn three_items_values_mut() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let values_mut = sparse_set.values_mut();
        assert_eq!(values_mut.len(), 3);
        assert_eq!(values_mut.into_slice(), &mut [34, 42, 69]);
    }

    #[test]
    fn three_items_into_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let values = sparse_set.into_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values.as_slice(), &[34, 42, 69]);
    }

    #[test]
    fn three_items_iter() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let iter = sparse_set.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.as_keys_slice(), &[2, 1, 5]);
        assert_eq!(iter.as_values_slice(), &[34, 42, 69]);
    }

    #[test]
    fn three_items_iter_mut() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let iter_mut = sparse_set.iter_mut();
        assert_eq!(iter_mut.len(), 3);
        assert_eq!(iter_mut.as_keys_slice(), &[2, 1, 5]);
        assert_eq!(iter_mut.into_values_slice(), &mut [34, 42, 69]);
    }

    #[test]
    fn three_items_into_iter() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let into_iter = sparse_set.into_iter();
        assert_eq!(into_iter.len(), 3);
        assert_eq!(into_iter.as_keys_slice(), &[2, 1, 5]);
        assert_eq!(into_iter.as_values_slice(), &[34, 42, 69]);
    }

    #[test]
    fn five_items_remove_insert() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(2, 69);
        sparse_set.insert(3, 228);
        sparse_set.insert(4, 666);

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, 42);

        let key = 3;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, 666);

        let key = 2;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, 69);

        let key = 3;
        let value = 0;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let key = 2;
        let value = 1;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let key = 4;
        let value = 10;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn five_items_swap_remove_insert() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(2, 69);
        sparse_set.insert(3, 228);
        sparse_set.insert(4, 666);

        let key = 1;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, 42);

        let key = 3;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, 666);

        let key = 2;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, 69);

        let key = 3;
        let value = 0;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let key = 2;
        let value = 1;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let key = 4;
        let value = 10;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn five_items_retain() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(4, 69);
        sparse_set.insert(3, 228);
        sparse_set.insert(6, 666);

        sparse_set.retain(|key, _| key % 2 == 0);
        assert_eq!(sparse_set.len(), 3);
        assert_eq!(sparse_set.keys().as_slice(), &[8, 4, 6]);
        assert_eq!(sparse_set.values().as_slice(), &[34, 69, 666]);

        sparse_set.retain(|_, value| *value % 2 == 1);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.keys().as_slice(), &[4]);
        assert_eq!(sparse_set.values().as_slice(), &[69]);
    }

    #[test]
    fn five_items_drain() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(4, 69);
        sparse_set.insert(3, 228);
        sparse_set.insert(6, 666);

        let drain = sparse_set.drain();
        assert_eq!(drain.as_keys_slice(), &[8, 1, 4, 3, 6]);
        assert_eq!(drain.as_values_slice(), &[34, 42, 69, 228, 666]);

        forget(drain);
        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.sparse_len(), 0);
        assert_eq!(sparse_set.keys().as_slice(), &[]);
        assert_eq!(sparse_set.values().as_slice(), &[]);
    }

    #[test]
    fn five_items_truncate() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(4, 69);
        sparse_set.insert(3, 228);
        sparse_set.insert(6, 666);

        sparse_set.truncate(usize::MAX, 5);
        assert_eq!(sparse_set.sparse_len(), 5);
        assert_eq!(sparse_set.keys().as_slice(), &[1, 4, 3]);
        assert_eq!(sparse_set.values().as_slice(), &[42, 69, 228]);

        assert_eq!(sparse_set.get(1), Some(&42));
        assert_eq!(sparse_set.get(4), Some(&69));
        assert_eq!(sparse_set.get(3), Some(&228));

        sparse_set.truncate(1, usize::MAX);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.keys().as_slice(), &[1]);
        assert_eq!(sparse_set.values().as_slice(), &[42]);

        assert_eq!(sparse_set.get(1), Some(&42));
    }

    #[test]
    fn five_items_sort() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, 42);
        sparse_set.insert(1, 228);
        sparse_set.insert(4, 69);
        sparse_set.insert(3, 666);
        sparse_set.insert(6, 34);

        sparse_set.sort();
        assert_eq!(sparse_set.keys().as_slice(), &[6, 8, 4, 1, 3]);
        assert_eq!(sparse_set.values().as_slice(), &[34, 42, 69, 228, 666]);

        assert_eq!(sparse_set.get(8), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&228));
        assert_eq!(sparse_set.get(4), Some(&69));
        assert_eq!(sparse_set.get(3), Some(&666));
        assert_eq!(sparse_set.get(6), Some(&34));
    }

    #[test]
    fn five_items_sort_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, 42);
        sparse_set.insert(1, 228);
        sparse_set.insert(4, 69);
        sparse_set.insert(3, 666);
        sparse_set.insert(6, 34);

        sparse_set.sort_keys();
        assert_eq!(sparse_set.keys().as_slice(), &[1, 3, 4, 6, 8]);
        assert_eq!(sparse_set.values().as_slice(), &[228, 666, 69, 34, 42]);

        assert_eq!(sparse_set.get(8), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&228));
        assert_eq!(sparse_set.get(4), Some(&69));
        assert_eq!(sparse_set.get(3), Some(&666));
        assert_eq!(sparse_set.get(6), Some(&34));
    }

    #[test]
    fn five_items_sort_by() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, 42);
        sparse_set.insert(1, 228);
        sparse_set.insert(4, 69);
        sparse_set.insert(3, 666);
        sparse_set.insert(6, 34);

        sparse_set.sort_by(|(_, a), (_, b)| Ord::cmp(b, a));
        assert_eq!(sparse_set.keys().as_slice(), &[3, 1, 4, 8, 6]);
        assert_eq!(sparse_set.values().as_slice(), &[666, 228, 69, 42, 34]);

        assert_eq!(sparse_set.get(8), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&228));
        assert_eq!(sparse_set.get(4), Some(&69));
        assert_eq!(sparse_set.get(3), Some(&666));
        assert_eq!(sparse_set.get(6), Some(&34));
    }

    #[test]
    fn from_keys_values_iter() {
        let keys = [3, 10, 5, 10, 1, usize::MAX];
        let values = [34, 42, 69, 228, 666];

        let sparse_set: SparseSet<i32> = keys.into_iter().zip(values).collect();
        assert_eq!(sparse_set.len(), 4);
        assert_eq!(sparse_set.keys().as_slice(), &[3, 10, 5, 1]);
        assert_eq!(sparse_set.values().as_slice(), &[34, 228, 69, 666]);

        assert_eq!(sparse_set.get(3), Some(&34));
        assert_eq!(sparse_set.get(10), Some(&228));
        assert_eq!(sparse_set.get(5), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&666));
    }

    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn from_keys_values_iter_too_large_key() {
        let keys = [3, 10, 5, 10, 1, usize::MAX];
        let values = [34, 42, 69, 228, 666, 999];

        let sparse_set: SparseSet<i32> = keys.into_iter().zip(values).collect();
        assert_eq!(sparse_set.len(), 4);
        assert_eq!(sparse_set.keys().as_slice(), &[3, 10, 5, 1, usize::MAX]);
        assert_eq!(sparse_set.values().as_slice(), &[34, 228, 69, 666, 999]);

        assert_eq!(sparse_set.get(3), Some(&34));
        assert_eq!(sparse_set.get(10), Some(&228));
        assert_eq!(sparse_set.get(5), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&666));
        assert_eq!(sparse_set.get(usize::MAX), Some(&999));
    }

    #[test]
    fn from_values_iter() {
        let values = [34, 42, 69, 228, 666];
        let sparse_set: SparseSet<i32> = values.into_iter().collect();

        assert_eq!(sparse_set.len(), 5);
        assert_eq!(sparse_set.keys().as_slice(), &[0, 1, 2, 3, 4]);
        assert_eq!(sparse_set.values().as_slice(), &[34, 42, 69, 228, 666]);

        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), Some(&42));
        assert_eq!(sparse_set.get(2), Some(&69));
        assert_eq!(sparse_set.get(3), Some(&228));
        assert_eq!(sparse_set.get(4), Some(&666));
    }

    #[test]
    fn extend_keys_values() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let keys = [3, 0, 2, 8];
        let values = [228, 666, 42, 69];
        sparse_set.extend(keys.into_iter().zip(values));

        assert_eq!(sparse_set.keys().as_slice(), &[2, 1, 5, 3, 0, 8]);
        assert_eq!(sparse_set.values().as_slice(), &[42, 42, 69, 228, 666, 69]);
    }

    #[test]
    fn extend_values() {
        let mut sparse_set = SparseSet::<i32>::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(4, 69);

        let values = [228, 666, 201];
        sparse_set.extend(values);

        assert_eq!(sparse_set.keys().as_slice(), &[2, 1, 4, 0, 3, 5]);
        assert_eq!(sparse_set.values().as_slice(), &[34, 42, 69, 228, 666, 201]);
    }
}
