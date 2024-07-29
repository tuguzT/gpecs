//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
// TODO `#![no_std]` with `alloc` enabled

extern crate alloc;

use alloc::collections::TryReserveError;
use core::mem::replace;
use nonmax::NonMaxUsize;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SparseSet<T> {
    dense: Vec<SparseSetEntry<T>>,
    sparse: Vec<Option<NonMaxUsize>>,
}

impl<T> SparseSet<T> {
    pub const fn new() -> Self {
        let dense = Vec::new();
        let sparse = Vec::new();
        Self { dense, sparse }
    }

    pub fn with_capacity(dense: usize, sparse: usize) -> Self {
        let dense = Vec::with_capacity(dense);
        let sparse = Vec::with_capacity(sparse);
        Self { dense, sparse }
    }

    #[inline]
    pub fn with_capacity_dense(dense: usize) -> Self {
        Self::with_capacity(dense, 0)
    }

    #[inline]
    pub fn with_capacity_sparse(sparse: usize) -> Self {
        Self::with_capacity(0, sparse)
    }

    #[inline]
    pub fn with_capacity_all(capacity: usize) -> Self {
        Self::with_capacity(capacity, capacity)
    }

    pub fn len(&self) -> usize {
        let Self { dense, .. } = self;
        dense.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        let Self { dense, sparse } = self;
        usize::min(dense.capacity(), sparse.capacity())
    }

    pub fn capacity_dense(&self) -> usize {
        let Self { dense, .. } = self;
        dense.capacity()
    }

    pub fn capacity_sparse(&self) -> usize {
        let Self { sparse, .. } = self;
        sparse.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        let Self { dense, sparse } = self;
        dense.reserve(additional);
        sparse.reserve(additional);
    }

    pub fn reserve_dense(&mut self, additional: usize) {
        let Self { dense, .. } = self;
        dense.reserve(additional);
    }

    pub fn reserve_sparse(&mut self, additional: usize) {
        let Self { sparse, .. } = self;
        sparse.reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let Self { dense, sparse } = self;
        dense.reserve_exact(additional);
        sparse.reserve_exact(additional);
    }

    pub fn reserve_dense_exact(&mut self, additional: usize) {
        let Self { dense, .. } = self;
        dense.reserve_exact(additional);
    }

    pub fn reserve_sparse_exact(&mut self, additional: usize) {
        let Self { sparse, .. } = self;
        sparse.reserve_exact(additional);
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { dense, sparse } = self;
        dense.try_reserve(additional)?;
        sparse.try_reserve(additional)?;
        Ok(())
    }

    pub fn try_reserve_dense(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { dense, .. } = self;
        dense.try_reserve(additional)
    }

    pub fn try_reserve_sparse(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { sparse, .. } = self;
        sparse.try_reserve(additional)
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { dense, sparse } = self;
        dense.try_reserve_exact(additional)?;
        sparse.try_reserve_exact(additional)?;
        Ok(())
    }

    pub fn try_reserve_dense_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { dense, .. } = self;
        dense.try_reserve_exact(additional)
    }

    pub fn try_reserve_sparse_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { sparse, .. } = self;
        sparse.try_reserve_exact(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        let Self { dense, sparse } = self;
        dense.shrink_to_fit();
        sparse.shrink_to_fit();
    }

    pub fn shrink_to_fit_dense(&mut self) {
        let Self { dense, .. } = self;
        dense.shrink_to_fit();
    }

    pub fn shrink_to_fit_sparse(&mut self) {
        let Self { sparse, .. } = self;
        sparse.shrink_to_fit();
    }

    pub fn as_slice(&self) -> &[SparseSetEntry<T>] {
        let Self { dense, .. } = self;
        dense.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [SparseSetEntry<T>] {
        let Self { dense, .. } = self;
        dense.as_mut_slice()
    }

    pub fn insert(&mut self, key: usize, value: T) -> Option<T> {
        let Self { dense, sparse } = self;

        if sparse.len() <= key {
            sparse.resize(key + 1, None);
        }

        if let Some(entry_index) = sparse.get(key).cloned().flatten().map(usize::from) {
            let entry_value = dense
                .get_mut(entry_index)
                .expect("index from sparse should be in bounds of dense")
                .value_mut();
            let value = replace(entry_value, value);
            return Some(value);
        }

        dense.push(SparseSetEntry { key, value });
        sparse[key] = (dense.len() - 1).try_into().ok();
        None
    }

    pub fn swap(&mut self, first_key: usize, second_key: usize) {
        let Self { dense, sparse } = self;

        if first_key == second_key {
            return;
        }

        let first_index = sparse.get(first_key).cloned().flatten().map(usize::from);
        let second_index = sparse.get(second_key).cloned().flatten().map(usize::from);
        let (Some(first_index), Some(second_index)) = (first_index, second_index) else {
            return;
        };

        // Cannot safely take 2 mutable references from the same dense, so...
        // 1. Validate indices to the dense, returns if any of them is out of bounds
        if first_index >= dense.len() || second_index >= dense.len() {
            return;
        }
        // (as I remember, from the current point of execution, dense index checks could be optimized away)
        // 2. Swap entries by valid indices
        dense.swap(first_index, second_index);
        // 3. Restore keys of swapped entries (these keys point to the sparse)
        let temp = dense[first_index].key;
        dense[first_index].key = dense[second_index].key;
        dense[second_index].key = temp;
    }

    pub fn swap_remove(&mut self, key: usize) -> Option<T> {
        let Self { dense, sparse } = self;

        let entry_index = sparse.get_mut(key).cloned().flatten().map(usize::from)?;
        if entry_index >= dense.len() {
            return None;
        }

        let entry = dense.swap_remove(entry_index);
        debug_assert_eq!(key, entry.key);

        let SparseSetEntry { value, .. } = entry;
        sparse[dense.len()] = sparse[key];
        sparse[key] = None;
        Some(value)
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        let Self { dense, sparse } = self;

        let entry_index = sparse.get_mut(key).cloned().flatten().map(usize::from)?;
        if entry_index >= dense.len() {
            return None;
        }

        for entry in dense.iter_mut().skip(entry_index + 1) {
            let sparse_index = entry.key;
            sparse[sparse_index] = sparse[sparse_index]
                .map(usize::from)
                .and_then(|index| (index - 1).try_into().ok());
        }

        let entry = dense.remove(entry_index);
        debug_assert_eq!(key, entry.key);

        let SparseSetEntry { value, .. } = entry;
        sparse[key] = None;
        Some(value)
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        let Self { dense, sparse } = self;

        let entry_index = sparse.get(key).cloned().flatten().map(usize::from)?;
        let entry = dense.get(entry_index)?;
        debug_assert_eq!(key, entry.key);

        let SparseSetEntry { value, .. } = entry;
        Some(value)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        let Self { dense, sparse } = self;

        let entry_index = sparse.get(key).cloned().flatten().map(usize::from)?;
        let entry = dense.get_mut(entry_index)?;
        debug_assert_eq!(key, entry.key);

        let SparseSetEntry { value, .. } = entry;
        Some(value)
    }

    pub fn contains(&self, key: usize) -> bool {
        let Self { dense, sparse } = self;

        let Some(entry_index) = sparse.get(key).cloned().flatten().map(usize::from) else {
            return false;
        };
        entry_index < dense.len()
    }

    pub fn clear(&mut self) {
        let Self { dense, sparse } = self;
        dense.clear();
        sparse.clear();
    }
}

// TODO FromIterator, IntoIterator, Extend

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SparseSetEntry<T> {
    key: usize,
    value: T,
}

impl<T> SparseSetEntry<T> {
    pub const fn key(&self) -> usize {
        let &Self { key, .. } = self;
        key
    }

    pub const fn value(&self) -> &T {
        let Self { value, .. } = self;
        value
    }

    pub fn value_mut(&mut self) -> &mut T {
        let Self { value, .. } = self;
        value
    }

    pub fn into_key(self) -> usize {
        let Self { key, .. } = self;
        key
    }

    pub fn into_value(self) -> T {
        let Self { value, .. } = self;
        value
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Not;

    use crate::SparseSet;

    #[test]
    fn empty() {
        let sparse_set = SparseSet::<i32>::new();
        assert!(sparse_set.is_empty());
        assert!(sparse_set.as_slice().is_empty());
    }

    #[test]
    fn with_capacity() {
        let sparse_set = SparseSet::<i32>::with_capacity_all(10);
        assert!(sparse_set.is_empty());
        assert!(sparse_set.as_slice().is_empty());
        assert_eq!(sparse_set.capacity_dense(), 10);
        assert_eq!(sparse_set.capacity_sparse(), 10);
    }

    #[test]
    fn empty_insert_one() {
        let mut sparse_set = SparseSet::new();
        let inserted = sparse_set.insert(0, 42);
        assert_eq!(inserted, None);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains(0));
    }

    #[test]
    fn with_capacity_insert_one() {
        let mut sparse_set = SparseSet::with_capacity_all(10);
        let inserted = sparse_set.insert(0, 42);
        assert_eq!(inserted, None);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains(0));
    }

    #[test]
    fn empty_insert_one_mutate() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        *sparse_set.get_mut(0).unwrap() = 43;

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&43));
        assert!(sparse_set.contains(0));
    }

    #[test]
    fn with_capacity_insert_one_mutate() {
        let mut sparse_set = SparseSet::with_capacity_all(10);
        sparse_set.insert(0, 42);
        *sparse_set.get_mut(0).unwrap() = 43;

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&43));
        assert!(sparse_set.contains(0));
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

        let inserted = sparse_set.insert(0, 34);
        assert_eq!(inserted, Some(42));

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

        let inserted = sparse_set.insert(1, 34);
        assert_eq!(inserted, Some(69));

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
}
