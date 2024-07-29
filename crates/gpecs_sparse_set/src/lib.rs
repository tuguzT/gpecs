//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{collections::TryReserveError, vec::Vec};
use core::{
    mem::{replace, swap},
    ops::{Index, IndexMut},
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

    pub fn dense_index(&self) -> Option<usize> {
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
    pub const fn new() -> Self {
        Self {
            dense_keys: Vec::new(),
            dense_values: Vec::new(),
            sparse: Vec::new(),
        }
    }

    pub fn with_capacity(dense: usize, sparse: usize) -> Self {
        Self {
            dense_keys: Vec::with_capacity(dense),
            dense_values: Vec::with_capacity(dense),
            sparse: Vec::with_capacity(sparse),
        }
    }

    #[inline(always)]
    pub fn with_dense_capacity(dense: usize) -> Self {
        Self::with_capacity(dense, 0)
    }

    #[inline(always)]
    pub fn with_sparse_capacity(sparse: usize) -> Self {
        Self::with_capacity(0, sparse)
    }

    #[inline(always)]
    pub fn with_capacity_all(capacity: usize) -> Self {
        Self::with_capacity(capacity, capacity)
    }

    pub fn len(&self) -> usize {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        debug_assert_eq!(dense_keys.len(), dense_values.len());
        dense_keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn sparse_len(&self) -> usize {
        let Self { sparse, .. } = self;
        sparse.len()
    }

    pub fn capacity(&self) -> usize {
        let dense_capacity = self.dense_capacity();
        let sparse_capacity = self.sparse_capacity();

        usize::min(dense_capacity, sparse_capacity)
    }

    pub fn dense_capacity(&self) -> usize {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        debug_assert_eq!(dense_keys.capacity(), dense_values.capacity());
        dense_keys.capacity()
    }

    pub fn sparse_capacity(&self) -> usize {
        let Self { sparse, .. } = self;
        sparse.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.reserve(additional);
        dense_values.reserve(additional);
        sparse.reserve(additional);
    }

    pub fn dense_reserve(&mut self, additional: usize) {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        dense_keys.reserve(additional);
        dense_values.reserve(additional);
    }

    pub fn sparse_reserve(&mut self, additional: usize) {
        let Self { sparse, .. } = self;
        sparse.reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.reserve_exact(additional);
        dense_values.reserve_exact(additional);
        sparse.reserve_exact(additional);
    }

    pub fn dense_reserve_exact(&mut self, additional: usize) {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        dense_keys.reserve_exact(additional);
        dense_values.reserve_exact(additional);
    }

    pub fn sparse_reserve_exact(&mut self, additional: usize) {
        let Self { sparse, .. } = self;
        sparse.reserve_exact(additional);
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.try_reserve(additional)?;
        dense_values.try_reserve(additional)?;
        sparse.try_reserve(additional)?;
        Ok(())
    }

    pub fn dense_try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        dense_keys.try_reserve(additional)?;
        dense_values.try_reserve(additional)?;
        Ok(())
    }

    pub fn sparse_try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { sparse, .. } = self;
        sparse.try_reserve(additional)
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        dense_keys.try_reserve_exact(additional)?;
        dense_values.try_reserve_exact(additional)?;
        sparse.try_reserve_exact(additional)?;
        Ok(())
    }

    pub fn dense_try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        dense_keys.try_reserve_exact(additional)?;
        dense_values.try_reserve_exact(additional)?;
        Ok(())
    }

    pub fn sparse_try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { sparse, .. } = self;
        sparse.try_reserve_exact(additional)
    }

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

    pub fn dense_shrink_to_fit(&mut self) {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        dense_keys.shrink_to_fit();
        dense_values.shrink_to_fit();
    }

    pub fn sparse_shrink_to_fit(&mut self) {
        let Self { sparse, .. } = self;
        sparse.shrink_to_fit();
    }

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

    pub fn dense_shrink_to(&mut self, min_capacity: usize) {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        dense_keys.shrink_to(min_capacity);
        dense_values.shrink_to(min_capacity);
    }

    pub fn sparse_shrink_to(&mut self, min_capacity: usize) {
        let Self { sparse, .. } = self;
        sparse.shrink_to(min_capacity);
    }

    pub fn as_slice(&self) -> &[T] {
        let Self { dense_values, .. } = self;
        dense_values.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let Self { dense_values, .. } = self;
        dense_values.as_mut_slice()
    }

    pub fn as_keys_slice(&self) -> &[usize] {
        let Self { dense_keys, .. } = self;
        dense_keys.as_slice()
    }

    pub fn insert(&mut self, key: usize, value: T) -> Option<T> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        if key >= sparse.len() {
            sparse.resize(key + 1, SparseEntry::Vacant);
        }

        if let SparseEntry::Occupied { dense_index } = sparse[key] {
            let entry_value = dense_values
                .get_mut(dense_index)
                .expect("index from sparse should be in bounds of dense");
            let value = replace(entry_value, value);
            return Some(value);
        }

        dense_keys.push(key);
        dense_values.push(value);
        sparse[key] = SparseEntry::occupied(dense_keys.len() - 1);

        None
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

        let first_index = sparse.get(first_key).and_then(SparseEntry::dense_index);
        let second_index = sparse.get(second_key).and_then(SparseEntry::dense_index);
        let (Some(first_index), Some(second_index)) = (first_index, second_index) else {
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

        let value = dense_values.remove(dense_index);
        let dense_key = dense_keys.remove(dense_index);
        debug_assert_eq!(key, dense_key);

        for sparse_index in dense_keys.iter().copied().skip(dense_index) {
            let sparse_entry = sparse
                .get_mut(sparse_index)
                .expect("key from dense should be in bounds of sparse");
            let dense_index = sparse_entry
                .dense_index_mut()
                .expect("current sparse entry should be occupied");
            *dense_index -= 1;
        }
        sparse[key] = SparseEntry::Vacant;

        Some(value)
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

    pub fn contains(&self, key: usize) -> bool {
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
    // TODO `iter`, `iter_mut`, `keys`, `values`, `values_mut`
}

impl<T> Index<usize> for SparseSet<T> {
    type Output = T;

    fn index(&self, key: usize) -> &Self::Output {
        match self.get(key) {
            Some(value) => value,
            None => panic!("key {key} not found"),
        }
    }
}

impl<T> IndexMut<usize> for SparseSet<T> {
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        match self.get_mut(key) {
            Some(value) => value,
            None => panic!("key {key} not found"),
        }
    }
}

impl<T> AsRef<[T]> for SparseSet<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for SparseSet<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

// TODO `FromIterator`, `IntoIterator`, `Extend`

#[cfg(test)]
mod tests {
    use std::ops::Not;

    use crate::SparseSet;

    #[test]
    fn empty() {
        let sparse_set = SparseSet::<i32>::new();
        assert!(sparse_set.is_empty());
    }

    #[test]
    fn with_capacity() {
        let sparse_set = SparseSet::<i32>::with_capacity_all(10);
        assert!(sparse_set.is_empty());
        assert_eq!(sparse_set.dense_capacity(), 10);
        assert_eq!(sparse_set.sparse_capacity(), 10);
    }

    #[test]
    fn empty_insert_one() {
        let mut sparse_set = SparseSet::new();
        let previous = sparse_set.insert(0, 42);
        assert_eq!(previous, None);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains(0));
    }

    #[test]
    fn with_capacity_insert_one() {
        let mut sparse_set = SparseSet::with_capacity_all(10);
        let previous = sparse_set.insert(0, 42);
        assert_eq!(previous, None);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains(0));
    }

    #[test]
    fn empty_insert_one_mutate() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set[0] = 43;

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&43));
        assert!(sparse_set.contains(0));
    }

    #[test]
    fn with_capacity_insert_one_mutate() {
        let mut sparse_set = SparseSet::with_capacity_all(10);
        sparse_set.insert(0, 42);
        sparse_set[0] = 43;

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&43));
        assert!(sparse_set.contains(0));
    }

    #[test]
    fn empty_insert_far() {
        let mut sparse_set = SparseSet::new();

        let (key, value) = (3, 42);
        sparse_set.insert(key, value);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let (key, value) = (6, 69);
        sparse_set.insert(key, value);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
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
        assert!(sparse_set.contains(key).not());

        let key = 1;
        let value = sparse_set.remove(key).unwrap();

        assert_eq!(value, 69);
        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains(key).not());
    }

    #[test]
    fn one_item_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(0), None);
        assert!(sparse_set.contains(0).not());
    }

    #[test]
    fn one_item_swap_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(0), None);
        assert!(sparse_set.contains(0).not());
    }

    #[test]
    fn one_item_swap() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        sparse_set.swap(0, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains(0));

        sparse_set.swap(0, 1);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains(0));
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
        assert!(sparse_set.contains(0));
        assert!(sparse_set.contains(1));
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
        assert!(sparse_set.contains(0));
        assert!(sparse_set.contains(1));
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
        assert!(sparse_set.contains(0).not());
        assert!(sparse_set.contains(1));
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
        assert!(sparse_set.contains(0).not());
        assert!(sparse_set.contains(1));
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
        assert!(sparse_set.contains(0));
        assert!(sparse_set.contains(1).not());
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
        assert!(sparse_set.contains(0));
        assert!(sparse_set.contains(1).not());
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
        assert!(sparse_set.contains(0));
        assert!(sparse_set.contains(1));
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
        assert!(sparse_set.contains(0));
        assert!(sparse_set.contains(1));
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
        assert!(sparse_set.contains(0));
        assert!(sparse_set.contains(1).not());
        assert!(sparse_set.contains(2));
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
        assert!(sparse_set.contains(0));
        assert!(sparse_set.contains(1).not());
        assert!(sparse_set.contains(2));
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
        assert!(sparse_set.contains(key));

        let key = 2;
        let value = 1;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let key = 4;
        let value = 10;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
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
        assert!(sparse_set.contains(key));

        let key = 2;
        let value = 1;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let key = 4;
        let value = 10;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }
}
