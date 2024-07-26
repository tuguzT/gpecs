//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
// TODO `#![no_std]` with `alloc` enabled

extern crate alloc;

use alloc::collections::TryReserveError;
use core::mem::replace;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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
    Free { next_free: usize },
}

impl Slot {
    pub fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied { .. })
    }

    pub fn is_free(&self) -> bool {
        matches!(self, Self::Free { .. })
    }

    pub fn dense_index(&self) -> Option<usize> {
        match self {
            Self::Occupied { dense_index } => Some(*dense_index),
            Self::Free { .. } => None,
        }
    }

    pub fn dense_index_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { dense_index } => Some(dense_index),
            Self::Free { .. } => None,
        }
    }

    pub fn next_free(&self) -> Option<usize> {
        match self {
            Self::Occupied { .. } => None,
            Self::Free { next_free } => Some(*next_free),
        }
    }

    pub fn next_free_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { .. } => None,
            Self::Free { next_free } => Some(next_free),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SparseSet<T> {
    dense: Vec<Entry<T>>,
    sparse: Vec<Slot>,
    first_free: usize,
    last_free: usize,
}

impl<T> SparseSet<T> {
    pub const fn new() -> Self {
        Self {
            dense: Vec::new(),
            sparse: Vec::new(),
            first_free: 0,
            last_free: 0,
        }
    }

    pub fn with_capacity(dense: usize, sparse: usize) -> Self {
        Self {
            dense: Vec::with_capacity(dense),
            sparse: Vec::with_capacity(sparse),
            first_free: 0,
            last_free: 0,
        }
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
        let Self { dense, sparse, .. } = self;
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
        let Self { dense, sparse, .. } = self;
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
        let Self { dense, sparse, .. } = self;
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
        let Self { dense, sparse, .. } = self;
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
        let Self { dense, sparse, .. } = self;
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
        let Self { dense, sparse, .. } = self;
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

    pub fn as_slice(&self) -> &[Entry<T>] {
        let Self { dense, .. } = self;
        dense.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [Entry<T>] {
        let Self { dense, .. } = self;
        dense.as_mut_slice()
    }

    pub fn insert(&mut self, key: usize, value: T) -> Option<T> {
        let Self {
            dense,
            sparse,
            first_free,
            last_free,
        } = self;

        if key >= sparse.len() {
            let sparse_len = sparse.len();
            if let Some(last_slot) = sparse.get_mut(*last_free) {
                let Slot::Free { next_free } = last_slot else {
                    panic!("last free should point to free slot");
                };
                *next_free = sparse_len;
            }

            let mut next_free = sparse_len;
            let generator = || {
                next_free = if next_free < key {
                    next_free + 1
                } else {
                    *first_free
                };
                Slot::Free { next_free }
            };
            sparse.resize_with(key + 1, generator);
        }

        match sparse[key] {
            Slot::Occupied { dense_index } => {
                let entry_value = dense
                    .get_mut(dense_index)
                    .expect("index from sparse should be in bounds of dense")
                    .value_mut();
                let value = replace(entry_value, value);
                Some(value)
            }
            Slot::Free { next_free } => {
                let entry = Entry { key, value };
                let slot = Slot::Occupied {
                    dense_index: dense.len(),
                };
                dense.push(entry);
                sparse[key] = slot;

                let (left, right) = sparse.split_at_mut(key);
                let left_free_slot = left
                    .iter_mut()
                    .enumerate()
                    .skip(*first_free)
                    .rfind(|(_, slot)| slot.is_free());
                let right_free_slot = right
                    .iter_mut()
                    .enumerate()
                    .map(|(idx, slot)| (idx + key, slot))
                    .skip(1)
                    .find(|(_, slot)| slot.is_free());
                match (left_free_slot, right_free_slot) {
                    (Some((_, Slot::Free { next_free: left })), Some((_, Slot::Free { .. }))) => {
                        *left = next_free;
                    }
                    (Some((left_index, Slot::Free { next_free: left })), None) => {
                        *last_free = left_index;
                        *left = next_free;
                    }
                    (None, Some((_, Slot::Free { .. }))) => {
                        *first_free = next_free;
                        if let Some(to_first_free) = sparse[*last_free].next_free_mut() {
                            *to_first_free = *first_free;
                        } else {
                            panic!("last free should point to free slot");
                        }
                    }
                    (None, None) => {
                        *first_free = sparse.len();
                        *last_free = sparse.len();
                    }
                    _ => unreachable!("found slot should be free"),
                }

                None
            }
        }
    }

    pub fn push(&mut self, value: T) -> usize {
        let Self {
            sparse,
            dense,
            first_free,
            last_free,
        } = self;

        if *first_free < sparse.len() {
            let key = *first_free;
            let entry = Entry { key, value };
            let slot = Slot::Occupied {
                dense_index: dense.len(),
            };

            let next_free = sparse[*first_free]
                .next_free()
                .expect("first free should point to free slot");
            let next_next_free = sparse
                .get_mut(next_free)
                .expect("next free should point to valid slot")
                .next_free_mut()
                .expect("next free should point to free slot");
            if *next_next_free == *first_free {
                *next_next_free = next_free;
            }

            dense.push(entry);
            sparse[*first_free] = slot;
            if *first_free == next_free {
                *first_free = sparse.len();
                *last_free = sparse.len();
            } else {
                if let Some(to_first_free) = sparse[*last_free].next_free_mut() {
                    *to_first_free = next_free;
                } else {
                    panic!("last free should point to free slot");
                }
                *first_free = next_free;
            }
            return key;
        }

        let key = *first_free;
        let entry = Entry { key, value };
        let slot = Slot::Occupied {
            dense_index: dense.len(),
        };
        dense.push(entry);
        sparse.push(slot);
        *first_free = sparse.len();
        *last_free = sparse.len();
        key
    }

    pub fn swap(&mut self, first_key: usize, second_key: usize) {
        let Self { dense, sparse, .. } = self;

        if first_key == second_key {
            return;
        }

        let first_index = sparse.get(first_key).and_then(Slot::dense_index);
        let second_index = sparse.get(second_key).and_then(Slot::dense_index);
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
        let Self {
            dense,
            sparse,
            first_free,
            last_free,
        } = self;

        let dense_index = sparse.get(key).and_then(Slot::dense_index)?;
        if dense_index >= dense.len() {
            panic!("index from sparse should be in bounds of dense");
        }

        let entry = dense.swap_remove(dense_index);
        debug_assert_eq!(key, entry.key);

        if let Some(entry) = dense.get(dense_index) {
            let slot = sparse
                .get_mut(entry.key)
                .expect("key from dense should point to valid sparse slot");
            let Slot::Occupied { dense_index: to_sw } = slot else {
                panic!("key from dense should point to occupied sparse slot");
            };
            debug_assert_eq!(*to_sw, entry.key);
            *to_sw = dense_index;
        }

        let next_free = match sparse.get_mut(*first_free) {
            Some(Slot::Free { next_free }) if *next_free == *first_free => {
                let result = *next_free;
                *next_free = key;
                *first_free = usize::min(key, *first_free);
                *last_free = usize::max(key, *last_free);
                result
            }
            Some(Slot::Free { .. }) => {
                let (left, right) = sparse.split_at_mut(key);
                let left_free_slot = left
                    .iter_mut()
                    .enumerate()
                    .skip(*first_free)
                    .rfind(|(_, slot)| slot.is_free());
                let right_free_slot = right
                    .iter_mut()
                    .enumerate()
                    .map(|(idx, slot)| (idx + key, slot))
                    .skip(1)
                    .find(|(_, slot)| slot.is_free());
                match (left_free_slot, right_free_slot) {
                    (
                        Some((_, Slot::Free { next_free: left })),
                        Some((right_index, Slot::Free { .. })),
                    ) => {
                        *left = key;
                        right_index
                    }
                    (None, Some((right_index, Slot::Free { .. }))) => {
                        *first_free = key;
                        if let Some(to_first_free) = sparse[*last_free].next_free_mut() {
                            *to_first_free = *first_free;
                        } else {
                            panic!("last free should point to free slot");
                        }
                        right_index
                    }
                    (Some((_, Slot::Free { next_free: left })), None) => {
                        *left = key;
                        *last_free = key;
                        *first_free
                    }
                    _ => unreachable!("found slot should be free"),
                }
            }
            Some(Slot::Occupied { .. }) => panic!("first free should point to free slot"),
            None => {
                *first_free = key;
                *last_free = key;
                key
            }
        };
        sparse[key] = Slot::Free { next_free };

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        let Self {
            dense,
            sparse,
            first_free,
            last_free,
        } = self;

        let dense_index = sparse.get(key).and_then(Slot::dense_index)?;
        if dense_index >= dense.len() {
            panic!("index from sparse should be in bounds of dense");
        }

        for entry in dense.iter_mut().skip(dense_index + 1) {
            let sparse_index = entry.key;
            if let Some(Slot::Occupied { dense_index }) = sparse.get_mut(sparse_index) {
                *dense_index -= 1;
            }
        }

        let entry = dense.remove(dense_index);
        debug_assert_eq!(key, entry.key);

        let next_free = match sparse.get_mut(*first_free) {
            Some(Slot::Free { next_free }) if *next_free == *first_free => {
                let result = *next_free;
                *next_free = key;
                *first_free = usize::min(key, *first_free);
                *last_free = usize::max(key, *last_free);
                result
            }
            Some(Slot::Free { .. }) => {
                let (left, right) = sparse.split_at_mut(key);
                let left_free_slot = left
                    .iter_mut()
                    .enumerate()
                    .skip(*first_free)
                    .rfind(|(_, slot)| slot.is_free());
                let right_free_slot = right
                    .iter_mut()
                    .enumerate()
                    .map(|(idx, slot)| (idx + key, slot))
                    .skip(1)
                    .find(|(_, slot)| slot.is_free());
                match (left_free_slot, right_free_slot) {
                    (
                        Some((_, Slot::Free { next_free: left })),
                        Some((right_index, Slot::Free { .. })),
                    ) => {
                        *left = key;
                        right_index
                    }
                    (None, Some((right_index, Slot::Free { .. }))) => {
                        *first_free = key;
                        if let Some(to_first_free) = sparse[*last_free].next_free_mut() {
                            *to_first_free = *first_free;
                        } else {
                            panic!("last free should point to free slot");
                        }
                        right_index
                    }
                    (Some((_, Slot::Free { next_free: left })), None) => {
                        *left = key;
                        *last_free = key;
                        *first_free
                    }
                    _ => unreachable!("found slot should be free"),
                }
            }
            Some(Slot::Occupied { .. }) => panic!("first free should point to free slot"),
            None => {
                *first_free = key;
                *last_free = key;
                key
            }
        };
        sparse[key] = Slot::Free { next_free };

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        let Self { dense, sparse, .. } = self;

        let slot = sparse.get(key).copied()?;
        let dense_index = slot.dense_index()?;
        let entry = dense.get(dense_index)?;
        debug_assert_eq!(key, entry.key);

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        let Self { dense, sparse, .. } = self;

        let slot = sparse.get(key).copied()?;
        let dense_index = slot.dense_index()?;
        let entry = dense.get_mut(dense_index)?;
        debug_assert_eq!(key, entry.key);

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn contains(&self, key: usize) -> bool {
        let Self { dense, sparse, .. } = self;

        let Some(slot) = sparse.get(key).copied() else {
            return false;
        };
        let Slot::Occupied { dense_index } = slot else {
            return false;
        };
        dense_index < dense.len()
    }

    pub fn clear(&mut self) {
        let Self {
            dense,
            sparse,
            first_free,
            last_free,
        } = self;

        dense.clear();
        sparse.clear();
        *first_free = 0;
        *last_free = 0;
    }
}

// TODO FromIterator, IntoIterator, Extend

#[cfg(test)]
mod tests {
    use core::{fmt::Debug, ops::Not};

    use crate::{Slot, SparseSet};

    fn debug_invariants<T>(sparse_set: &SparseSet<T>, message: impl AsRef<str>)
    where
        T: Debug,
    {
        let message = message.as_ref();
        let message = if message.is_empty() {
            "Validating sparse set"
        } else {
            message
        };
        println!("{message}:\n{sparse_set:#?}\n");

        assert_invariants(sparse_set);
    }

    fn assert_invariants<T>(sparse_set: &SparseSet<T>) {
        let SparseSet {
            dense,
            sparse,
            first_free,
            last_free,
        } = sparse_set;

        let mut calculated_first_free = None;
        let mut calculated_last_free = None;
        for (key, slot) in sparse.iter().copied().enumerate() {
            match slot {
                Slot::Occupied { dense_index } => {
                    let entry = dense
                        .get(dense_index)
                        .expect("index from sparse should be in bounds of dense");
                    assert_eq!(
                        entry.key, key,
                        "key from dense should be equal to sparse index",
                    );
                }
                Slot::Free { next_free } => {
                    if calculated_first_free.is_none() {
                        assert_eq!(
                            *first_free, key,
                            "first free should point to the first free slot",
                        );
                        calculated_first_free = Some(key);
                    }

                    // TODO check `prev_free` of the current free slot

                    if key < next_free && sparse[(key + 1)..next_free].iter().any(Slot::is_free) {
                        panic!("there should be no free slots between the current free slot and next free slot");
                    }
                    let Slot::Free { .. } = sparse[next_free] else {
                        panic!("next free should point to free slot");
                    };

                    if let Some(prev_free_slot) = calculated_last_free.map(|key| sparse[key]) {
                        let Slot::Free { next_free } = prev_free_slot else {
                            panic!("next free should point to free slot");
                        };
                        assert_eq!(
                            next_free, key,
                            "next free of previous free slot should point to the current free slot",
                        );
                    }
                    calculated_last_free = Some(key);
                }
            }
        }

        match (calculated_first_free, calculated_last_free) {
            (Some(calculated_first_free), Some(calculated_last_free)) => {
                assert_eq!(
                    calculated_first_free, *first_free,
                    "first free should point to the first free slot",
                );
                let Slot::Free { .. } = sparse[calculated_first_free] else {
                    panic!("first free should point to free slot")
                };
                // TODO assert equality of first free slot's `prev_free` and `last_free`

                assert_eq!(
                    calculated_last_free, *last_free,
                    "last free should point to the last free slot",
                );
                let Slot::Free { next_free, .. } = sparse[calculated_last_free] else {
                    panic!("last free should point to free slot");
                };
                assert_eq!(
                    next_free, *first_free,
                    "last free slot should point to first free",
                );
            }
            (None, None) => {
                let sparse_len = sparse.len();
                assert_eq!(
                    *first_free, sparse_len,
                    "first free should point to the end of sparse if no free slots were found",
                );
                assert_eq!(
                    *last_free, sparse_len,
                    "last free should point to the end of sparse if no free slots were found",
                );
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn empty() {
        let sparse_set = SparseSet::<i32>::new();
        debug_invariants(&sparse_set, "Empty");

        assert!(sparse_set.is_empty());
        assert!(sparse_set.as_slice().is_empty());
    }

    #[test]
    fn with_capacity() {
        let sparse_set = SparseSet::<i32>::with_capacity_all(10);
        debug_invariants(&sparse_set, "Empty with capacity");

        assert!(sparse_set.is_empty());
        assert!(sparse_set.as_slice().is_empty());
        assert_eq!(sparse_set.capacity_dense(), 10);
        assert_eq!(sparse_set.capacity_sparse(), 10);
    }

    #[test]
    fn empty_insert_one() {
        let mut sparse_set = SparseSet::new();
        debug_invariants(&sparse_set, "Empty");

        let (key, value) = (0, 42);
        let previous = sparse_set.insert(key, value);
        assert_eq!(previous, None);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }

    #[test]
    fn with_capacity_insert_one() {
        let mut sparse_set = SparseSet::with_capacity_all(10);
        debug_invariants(&sparse_set, "Empty with capacity");

        let (key, value) = (0, 42);
        let previous = sparse_set.insert(key, value);
        assert_eq!(previous, None);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }

    #[test]
    fn empty_insert_one_mutate() {
        let mut sparse_set = SparseSet::new();
        debug_invariants(&sparse_set, "Empty");

        let (key, value) = (0, 42);
        sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        let value = 43;
        *sparse_set.get_mut(key).unwrap() = value;
        debug_invariants(&sparse_set, format!("Changed key {key} to value {value}"));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }

    #[test]
    fn with_capacity_insert_one_mutate() {
        let mut sparse_set = SparseSet::with_capacity_all(10);
        debug_invariants(&sparse_set, "Empty with capacity");

        let (key, value) = (0, 42);
        sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        let value = 43;
        *sparse_set.get_mut(key).unwrap() = value;
        debug_invariants(&sparse_set, format!("Changed key {key} to value {value}"));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }

    #[test]
    fn empty_insert_far() {
        let mut sparse_set = SparseSet::new();
        debug_invariants(&sparse_set, "Empty");

        let (key, value) = (3, 42);
        sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let (key, value) = (6, 69);
        sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }

    #[test]
    fn empty_insert_far_push() {
        let mut sparse_set = SparseSet::new();
        debug_invariants(&sparse_set, "Empty");

        let (key, value) = (4, 42);
        sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let value = 69;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 0);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }

    #[test]
    fn empty_insert_far_remove() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(3, 42);
        sparse_set.insert(1, 69);
        debug_invariants(&sparse_set, "Inserted two values");

        let key = 3;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains(key).not());

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 69);
        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains(key).not());
    }

    #[test]
    fn empty_push() {
        let mut sparse_set = SparseSet::new();
        debug_invariants(&sparse_set, "Empty");

        let value = 42;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 0);
        assert_eq!(sparse_set.get(key), Some(&value));

        let value = 69;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 1);
        assert_eq!(sparse_set.get(key), Some(&value));
    }

    #[test]
    fn one_item_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        debug_invariants(&sparse_set, "Inserted one value");

        let key = 0;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains(key).not());
    }

    #[test]
    fn one_item_swap_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        debug_invariants(&sparse_set, "Inserted one value");

        let key = 0;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains(key).not());
    }

    #[test]
    fn one_item_swap() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        debug_invariants(&sparse_set, "Inserted one value");

        sparse_set.swap(0, 0);
        debug_invariants(&sparse_set, "Swapped first with self");

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains(0));

        sparse_set.swap(0, 1);
        debug_invariants(&sparse_set, "Swapped first with non-existent key");

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains(0));
    }

    #[test]
    fn one_item_remove_push() {
        let mut sparse_set = SparseSet::new();

        let value = 42;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        let removed = sparse_set.remove(key);
        assert_eq!(removed, Some(value));
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains(key).not());
    }

    #[test]
    fn one_item_swap_remove_push() {
        let mut sparse_set = SparseSet::new();

        let value = 42;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        let removed = sparse_set.swap_remove(key);
        assert_eq!(removed, Some(value));
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains(key).not());
    }

    #[test]
    fn two_items_insert_first() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);
        debug_invariants(&sparse_set, "Inserted two values");

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let previous = sparse_set.insert(0, 34);
        assert_eq!(previous, Some(42));
        debug_invariants(&sparse_set, "Replaced key 0 of value 42 to value 34");

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
        debug_invariants(&sparse_set, "Inserted two values");

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let previous = sparse_set.insert(1, 34);
        assert_eq!(previous, Some(69));
        debug_invariants(&sparse_set, "Replaced key 1 of value 69 to value 34");

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
        debug_invariants(&sparse_set, "Inserted two values");

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some(42));
        debug_invariants(&sparse_set, "Removed key 0 of value 42");

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
        debug_invariants(&sparse_set, "Inserted two values");

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some(42));
        debug_invariants(&sparse_set, "Removed key 0 of value 42");

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
        debug_invariants(&sparse_set, "Inserted two values");

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let removed = sparse_set.remove(1);
        assert_eq!(removed, Some(69));
        debug_invariants(&sparse_set, "Removed key 1 of value 69");

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
        debug_invariants(&sparse_set, "Inserted two values");

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        let removed = sparse_set.swap_remove(1);
        assert_eq!(removed, Some(69));
        debug_invariants(&sparse_set, "Removed key 1 of value 69");

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
        debug_invariants(&sparse_set, "Inserted two values");

        let key = 0;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains(key).not());

        let value = 34;
        let previous = sparse_set.insert(key, value);
        assert_eq!(previous, None);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

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
        debug_invariants(&sparse_set, "Inserted two values");

        let key = 0;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains(key).not());

        let value = 34;
        let previous = sparse_set.insert(key, value);
        assert_eq!(previous, None);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

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
        debug_invariants(&sparse_set, "Inserted two values");

        sparse_set.swap(0, 0);
        debug_invariants(&sparse_set, "Swapped first with self");

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        sparse_set.swap(0, 1);
        debug_invariants(&sparse_set, "Swapped first with second");

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&42));

        sparse_set.swap(1, 1);
        debug_invariants(&sparse_set, "Swapped second with self");

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&42));
    }

    #[test]
    fn two_items_remove_first_push() {
        let mut sparse_set = SparseSet::new();
        sparse_set.push(42);
        sparse_set.push(69);
        debug_invariants(&sparse_set, "Pushed two values");

        let key = 0;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), None);

        let value = 34;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 0);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));
    }

    #[test]
    fn two_items_swap_remove_first_push() {
        let mut sparse_set = SparseSet::new();
        sparse_set.push(42);
        sparse_set.push(69);
        debug_invariants(&sparse_set, "Pushed two values");

        let key = 0;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), None);

        let value = 34;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 0);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));
    }

    #[test]
    fn two_items_remove_second_push() {
        let mut sparse_set = SparseSet::new();
        sparse_set.push(42);
        sparse_set.push(69);
        debug_invariants(&sparse_set, "Pushed two values");

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 69);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), None);

        let value = 34;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));
    }

    #[test]
    fn two_items_swap_remove_second_push() {
        let mut sparse_set = SparseSet::new();
        sparse_set.push(42);
        sparse_set.push(69);
        debug_invariants(&sparse_set, "Pushed two values");

        let key = 1;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 69);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), None);

        let value = 34;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));
    }

    #[test]
    fn two_items_remove_all_push() {
        let mut sparse_set = SparseSet::new();
        sparse_set.push(42);
        sparse_set.push(69);
        debug_invariants(&sparse_set, "Pushed two values");

        let key = 0;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), None);

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 69);
        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);

        let value = 34;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));

        let value = 228;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));

        let value = 0;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 2);
        assert_eq!(sparse_set.len(), 3);
        assert_eq!(sparse_set.get(key), Some(&value));
    }

    #[test]
    fn two_items_swap_remove_all_push() {
        let mut sparse_set = SparseSet::new();
        sparse_set.push(42);
        sparse_set.push(69);
        debug_invariants(&sparse_set, "Pushed two values");

        let key = 0;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), None);

        let key = 1;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 69);
        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);

        let value = 34;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some(&value));

        let value = 228;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some(&value));

        let value = 0;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 2);
        assert_eq!(sparse_set.len(), 3);
        assert_eq!(sparse_set.get(key), Some(&value));
    }

    #[test]
    fn three_items_remove_middle() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(2, 69);
        debug_invariants(&sparse_set, "Inserted three values");

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
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
        debug_invariants(&sparse_set, "Inserted three values");

        let key = 1;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), None);
        assert_eq!(sparse_set.get(2), Some(&69));
        assert!(sparse_set.contains(0));
        assert!(sparse_set.contains(1).not());
        assert!(sparse_set.contains(2));
    }

    #[test]
    fn three_items_remove_middle_push() {
        let mut sparse_set = SparseSet::new();
        sparse_set.push(34);
        sparse_set.push(42);
        sparse_set.push(69);
        debug_invariants(&sparse_set, "Inserted three values");

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), None);
        assert_eq!(sparse_set.get(2), Some(&69));

        let value = 228;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 1);
        assert_eq!(sparse_set.len(), 3);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), Some(&228));
        assert_eq!(sparse_set.get(2), Some(&69));
    }

    #[test]
    fn three_items_swap_remove_middle_push() {
        let mut sparse_set = SparseSet::new();
        sparse_set.push(34);
        sparse_set.push(42);
        sparse_set.push(69);
        debug_invariants(&sparse_set, "Inserted three values");

        let key = 1;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));

        assert_eq!(value, 42);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), None);
        assert_eq!(sparse_set.get(2), Some(&69));

        let value = 228;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 1);
        assert_eq!(sparse_set.len(), 3);
        assert_eq!(sparse_set.get(0), Some(&34));
        assert_eq!(sparse_set.get(1), Some(&228));
        assert_eq!(sparse_set.get(2), Some(&69));
    }

    #[test]
    fn five_items_remove_insert() {
        let mut sparse_set = SparseSet::new();
        assert_eq!(sparse_set.push(34), 0);
        assert_eq!(sparse_set.push(42), 1);
        assert_eq!(sparse_set.push(69), 2);
        assert_eq!(sparse_set.push(228), 3);
        assert_eq!(sparse_set.push(666), 4);
        debug_invariants(&sparse_set, "Inserted five values");

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 42);

        let key = 3;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 666);

        let key = 2;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 69);

        let key = 3;
        let value = 0;
        let previous = sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let key = 2;
        let value = 1;
        let previous = sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let key = 4;
        let value = 10;
        let previous = sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }

    #[test]
    fn five_items_swap_remove_insert() {
        let mut sparse_set = SparseSet::new();
        assert_eq!(sparse_set.push(34), 0);
        assert_eq!(sparse_set.push(42), 1);
        assert_eq!(sparse_set.push(69), 2);
        assert_eq!(sparse_set.push(228), 3);
        assert_eq!(sparse_set.push(666), 4);
        debug_invariants(&sparse_set, "Inserted five values");

        let key = 1;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 42);

        let key = 3;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 666);

        let key = 2;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 69);

        let key = 3;
        let value = 0;
        let previous = sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let key = 2;
        let value = 1;
        let previous = sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let key = 4;
        let value = 10;
        let previous = sparse_set.insert(key, value);
        debug_invariants(&sparse_set, format!("Inserted key {key} of value {value}"));

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }

    #[test]
    fn five_items_remove_push() {
        let mut sparse_set = SparseSet::new();
        assert_eq!(sparse_set.push(34), 0);
        assert_eq!(sparse_set.push(42), 1);
        assert_eq!(sparse_set.push(69), 2);
        assert_eq!(sparse_set.push(228), 3);
        assert_eq!(sparse_set.push(666), 4);
        debug_invariants(&sparse_set, "Inserted five values");

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 42);

        let key = 3;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 666);

        let key = 2;
        let value = sparse_set.remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 69);

        let value = 0;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let value = 1;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 2);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let value = 10;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 3);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }

    #[test]
    fn five_items_swap_remove_push() {
        let mut sparse_set = SparseSet::new();
        assert_eq!(sparse_set.push(34), 0);
        assert_eq!(sparse_set.push(42), 1);
        assert_eq!(sparse_set.push(69), 2);
        assert_eq!(sparse_set.push(228), 3);
        assert_eq!(sparse_set.push(666), 4);
        debug_invariants(&sparse_set, "Inserted five values");

        let key = 1;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 42);

        let key = 3;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 666);

        let key = 2;
        let value = sparse_set.swap_remove(key).unwrap();
        debug_invariants(&sparse_set, format!("Removed key {key} of value {value}"));
        assert_eq!(value, 69);

        let value = 0;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 1);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let value = 1;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 2);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));

        let value = 10;
        let key = sparse_set.push(value);
        debug_invariants(&sparse_set, format!("Pushed value {value}"));

        assert_eq!(key, 3);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains(key));
    }
}
