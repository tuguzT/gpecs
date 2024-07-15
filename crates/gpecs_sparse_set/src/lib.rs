//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
// TODO `#![no_std]` with `alloc` enabled

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

    pub fn with_capacity(capacity: usize) -> Self {
        let dense = Vec::with_capacity(capacity);
        let sparse = Vec::with_capacity(capacity);
        Self { dense, sparse }
    }

    pub fn capacity(&self) -> usize {
        let Self { dense, sparse } = self;
        debug_assert_eq!(dense.capacity(), sparse.capacity());
        dense.capacity()
    }

    // TODO more methods
}

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

    pub fn destruct(self) -> (usize, T) {
        let Self { key, value } = self;
        (key, value)
    }
}
