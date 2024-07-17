//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
// TODO `#![no_std]` with `alloc` enabled

use std::{collections::TryReserveError, mem::replace};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SparseSet<T> {
    dense: Vec<SparseSetEntry<T>>,
    sparse: Vec<usize>,
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

        if sparse.len() < key {
            sparse.resize(key, usize::MAX);
        }

        let entry_index = sparse.get_mut(key)?;
        match dense.get_mut(*entry_index) {
            Some(entry) => {
                let value = replace(&mut entry.value, value);
                Some(value)
            }
            None => {
                dense.push(SparseSetEntry { key, value });
                *entry_index = dense.len() - 1;
                None
            }
        }
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        let Self { dense, sparse } = self;

        let entry_index = sparse.get_mut(key).cloned()?;
        if entry_index < dense.len() {
            let value = dense.swap_remove(entry_index).value;
            dense.get_mut(entry_index)?.key = key;
            return Some(value);
        }
        None
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        let Self { dense, sparse } = self;

        let entry_index = sparse.get(key).cloned()?;
        let entry = dense.get(entry_index)?;
        debug_assert_eq!(key, entry.key);

        let SparseSetEntry { value, .. } = entry;
        Some(value)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        let Self { dense, sparse } = self;

        let entry_index = sparse.get(key).cloned()?;
        let entry = dense.get_mut(entry_index)?;
        debug_assert_eq!(key, entry.key);

        let SparseSetEntry { value, .. } = entry;
        Some(value)
    }

    pub fn contains(&self, key: usize) -> bool {
        let Self { dense, sparse } = self;

        let Some(entry_index) = sparse.get(key).cloned() else {
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
