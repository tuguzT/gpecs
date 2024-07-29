//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
// TODO `#![no_std]` with `alloc` enabled

extern crate alloc;

use alloc::collections::TryReserveError;
use core::mem::{replace, swap};

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
pub struct Entry<T> {
    key: usize,
    value: T,
}

impl<T> Entry<T> {
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Slot {
    Occupied { dense_index: usize },
    Free,
}

impl Slot {
    pub const fn occupied(dense_index: usize) -> Self {
        Self::Occupied { dense_index }
    }

    pub const fn free() -> Self {
        Self::Free
    }

    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied { .. })
    }

    pub const fn is_free(&self) -> bool {
        matches!(self, Self::Free)
    }

    pub fn dense_index(&self) -> Option<usize> {
        match self {
            Self::Occupied { dense_index } => Some(*dense_index),
            Self::Free => None,
        }
    }

    pub fn dense_index_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { dense_index } => Some(dense_index),
            Self::Free => None,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SparseSet<T> {
    dense: Vec<Entry<T>>,
    sparse: Vec<Slot>,
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

    #[inline(always)]
    pub fn with_capacity_dense(dense: usize) -> Self {
        Self::with_capacity(dense, 0)
    }

    #[inline(always)]
    pub fn with_capacity_sparse(sparse: usize) -> Self {
        Self::with_capacity(0, sparse)
    }

    #[inline(always)]
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

    pub fn insert(&mut self, key: usize, value: T) -> Option<T> {
        let Self { dense, sparse } = self;

        if key >= sparse.len() {
            sparse.resize(key + 1, Slot::Free);
        }

        if let Slot::Occupied { dense_index } = sparse[key] {
            let entry_value = dense
                .get_mut(dense_index)
                .expect("index from sparse should be in bounds of dense")
                .value_mut();
            let value = replace(entry_value, value);
            return Some(value);
        }

        let entry = Entry { key, value };
        let slot = Slot::occupied(dense.len());
        dense.push(entry);
        sparse[key] = slot;

        None
    }

    pub fn swap(&mut self, first_key: usize, second_key: usize) {
        let Self { dense, sparse } = self;

        if first_key == second_key {
            return;
        }

        let first_index = sparse.get(first_key).and_then(Slot::dense_index);
        let second_index = sparse.get(second_key).and_then(Slot::dense_index);
        let (Some(first_index), Some(second_index)) = (first_index, second_index) else {
            return;
        };

        let (first_entry, second_entry) = get_pair_mut(dense, first_index, second_index)
            .expect("indices from sparse should be in bounds of dense and differ from each other");
        let first_value = first_entry.value_mut();
        let second_value = second_entry.value_mut();
        swap(first_value, second_value);
    }

    pub fn swap_remove(&mut self, key: usize) -> Option<T> {
        let Self { dense, sparse } = self;

        let dense_index = sparse.get(key).and_then(Slot::dense_index)?;
        assert!(
            dense_index < dense.len(),
            "index from sparse should be in bounds of dense",
        );

        let entry = dense.swap_remove(dense_index);
        debug_assert_eq!(key, entry.key);

        sparse[dense.len()] = sparse[key];
        sparse[key] = Slot::Free;

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        let Self { dense, sparse } = self;

        let dense_index = sparse.get(key).and_then(Slot::dense_index)?;
        assert!(
            dense_index < dense.len(),
            "index from sparse should be in bounds of dense",
        );

        for entry in dense.iter_mut().skip(dense_index + 1) {
            let sparse_index = entry.key;
            let slot = sparse
                .get_mut(sparse_index)
                .expect("key from dense should be in bounds of sparse");
            let dense_index = slot
                .dense_index_mut()
                .expect("current slot should be occupied");
            *dense_index -= 1;
        }

        let entry = dense.remove(dense_index);
        debug_assert_eq!(key, entry.key);

        sparse[key] = Slot::Free;

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        let Self { dense, sparse } = self;

        let slot = sparse.get(key).copied()?;
        let dense_index = slot.dense_index()?;
        let entry = dense
            .get(dense_index)
            .expect("index from sparse should be in bounds of dense");
        debug_assert_eq!(key, entry.key);

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        let Self { dense, sparse } = self;

        let slot = sparse.get(key).copied()?;
        let dense_index = slot.dense_index()?;
        let entry = dense
            .get_mut(dense_index)
            .expect("index from sparse should be in bounds of dense");
        debug_assert_eq!(key, entry.key);

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn contains(&self, key: usize) -> bool {
        let Self { dense, sparse } = self;

        let Some(slot) = sparse.get(key).copied() else {
            return false;
        };
        let Slot::Occupied { dense_index } = slot else {
            return false;
        };

        debug_assert!(
            dense_index < dense.len(),
            "index from sparse should be in bounds of dense",
        );
        true
    }

    pub fn clear(&mut self) {
        let Self { dense, sparse } = self;
        dense.clear();
        sparse.clear();
    }
}

// TODO FromIterator, IntoIterator, Extend

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
        assert_eq!(sparse_set.capacity_dense(), 10);
        assert_eq!(sparse_set.capacity_sparse(), 10);
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
