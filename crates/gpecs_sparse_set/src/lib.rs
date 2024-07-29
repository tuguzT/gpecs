//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
// TODO `#![no_std]` with `alloc` enabled

extern crate alloc;

use alloc::collections::TryReserveError;
use core::mem::{replace, swap};

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

    pub fn into_parts(self) -> (usize, T) {
        let Self { key, value } = self;
        (key, value)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Slot {
    Occupied(OccupiedSlot),
    Free(FreeSlot),
}

impl Slot {
    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied { .. })
    }

    pub const fn is_free(&self) -> bool {
        matches!(self, Self::Free { .. })
    }

    pub const fn occupied(&self) -> Option<OccupiedSlot> {
        match self {
            Self::Occupied(occupied) => Some(*occupied),
            Self::Free(_) => None,
        }
    }

    pub fn occupied_mut(&mut self) -> Option<&mut OccupiedSlot> {
        match self {
            Self::Occupied(occupied) => Some(occupied),
            Self::Free(_) => None,
        }
    }

    pub const fn free(&self) -> Option<FreeSlot> {
        match self {
            Self::Occupied(_) => None,
            Self::Free(free) => Some(*free),
        }
    }

    pub fn free_mut(&mut self) -> Option<&mut FreeSlot> {
        match self {
            Self::Occupied(_) => None,
            Self::Free(free) => Some(free),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct OccupiedSlot {
    pub dense_index: usize,
}

impl OccupiedSlot {
    pub fn new(dense_index: usize) -> Self {
        Self { dense_index }
    }

    pub fn dense_index(&self) -> usize {
        let Self { dense_index } = self;
        *dense_index
    }

    pub fn dense_index_mut(&mut self) -> &mut usize {
        let Self { dense_index } = self;
        dense_index
    }

    pub fn into_dense_index(self) -> usize {
        let Self { dense_index } = self;
        dense_index
    }
}

impl From<OccupiedSlot> for Slot {
    fn from(occupied: OccupiedSlot) -> Self {
        Self::Occupied(occupied)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct FreeSlot {
    pub prev_free: usize,
    pub next_free: usize,
}

impl FreeSlot {
    pub fn new(prev_free: usize, next_free: usize) -> Self {
        Self {
            prev_free,
            next_free,
        }
    }

    pub fn next_free(&self) -> usize {
        let Self { next_free, .. } = self;
        *next_free
    }

    pub fn next_free_mut(&mut self) -> &mut usize {
        let Self { next_free, .. } = self;
        next_free
    }

    pub fn into_next_free(self) -> usize {
        let Self { next_free, .. } = self;
        next_free
    }
}

impl From<FreeSlot> for Slot {
    fn from(free: FreeSlot) -> Self {
        Self::Free(free)
    }
}

fn get_pair_mut<T>(slice: &mut [T], i: usize, j: usize) -> Option<(&mut T, &mut T)> {
    let (first, second) = (usize::min(i, j), usize::max(i, j));

    let [first, .., second] = slice.get_mut(first..=second)? else {
        return None;
    };

    let pair = if i < j {
        (first, second)
    } else {
        (second, first)
    };
    Some(pair)
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

    #[inline(always)]
    fn extend_sparse(&mut self, new_sparse_len: usize) {
        let Self {
            sparse,
            first_free,
            last_free,
            ..
        } = self;

        if new_sparse_len <= sparse.len() {
            return;
        }

        let sparse_len = sparse.len();
        if let Some(last_slot) = sparse.get_mut(*last_free) {
            let last_slot = last_slot
                .free_mut()
                .expect("last free should point to free slot");
            last_slot.next_free = sparse_len;
        }
        if let Some(first_slot) = sparse.get_mut(*first_free) {
            let first_slot = first_slot
                .free_mut()
                .expect("first free should point to free slot");
            first_slot.prev_free = new_sparse_len - 1;
        }

        let mut current = sparse_len;
        let generator = || {
            let prev_free = if current > sparse_len {
                current - 1
            } else if *last_free < sparse_len {
                *last_free
            } else {
                new_sparse_len - 1
            };
            let next_free = if current < new_sparse_len - 1 {
                current + 1
            } else {
                *first_free
            };
            let slot = FreeSlot::new(prev_free, next_free);

            current += 1;
            slot.into()
        };
        sparse.resize_with(new_sparse_len, generator);

        *last_free = new_sparse_len - 1;
    }

    #[inline(always)]
    fn replace_at(&mut self, key: usize, value: T) -> T {
        let Self { dense, sparse, .. } = self;
        let dense = dense.as_mut_slice();
        let sparse = sparse.as_mut_slice();

        let current_occupied_slot = sparse
            .get(key)
            .expect("current slot should be present")
            .occupied()
            .expect("current slot should be occupied");
        let OccupiedSlot { dense_index } = current_occupied_slot;

        let entry_value = dense
            .get_mut(dense_index)
            .expect("index from sparse should be in bounds of dense")
            .value_mut();
        replace(entry_value, value)
    }

    #[inline(always)]
    fn occupy_at(&mut self, key: usize, value: T) {
        let Self {
            dense,
            sparse,
            first_free,
            last_free,
        } = self;
        let sparse = sparse.as_mut_slice();

        let current_free_slot = sparse
            .get(key)
            .expect("current slot should be present")
            .free()
            .expect("current slot should be free");
        let FreeSlot {
            prev_free,
            next_free,
        } = current_free_slot;

        let entry = Entry { key, value };
        let slot = OccupiedSlot::new(dense.len()).into();
        dense.push(entry);
        sparse[key] = slot;

        // there is only one free slot, which is the current one
        if prev_free == next_free {
            *first_free = sparse.len();
            *last_free = sparse.len();
            return;
        }

        let (prev_free_slot, next_free_slot) = get_pair_mut(sparse, prev_free, next_free)
            .expect("prev free and next free should be in bounds of sparse");

        let prev_free_slot = prev_free_slot
            .free_mut()
            .expect("prev free should point to free slot");
        let next_free_slot = next_free_slot
            .free_mut()
            .expect("next free should point to free slot");

        prev_free_slot.next_free = next_free;
        next_free_slot.prev_free = prev_free;

        if key == *first_free {
            *first_free = next_free;
        }
        if key == *last_free {
            *last_free = prev_free;
        }
    }

    pub fn insert(&mut self, key: usize, value: T) -> Option<T> {
        self.extend_sparse(key + 1);

        let Self { sparse, .. } = self;
        let sparse = sparse.as_slice();

        match sparse[key] {
            Slot::Occupied(_) => {
                let value = self.replace_at(key, value);
                Some(value)
            }
            Slot::Free(_) => {
                self.occupy_at(key, value);
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

        let key = *first_free;
        if key < sparse.len() {
            self.occupy_at(key, value);
            return key;
        }

        let entry = Entry { key, value };
        let slot = OccupiedSlot::new(dense.len()).into();
        dense.push(entry);
        sparse.push(slot);

        *first_free = sparse.len();
        *last_free = sparse.len();

        key
    }

    pub fn swap(&mut self, first_key: usize, second_key: usize) {
        let Self { dense, sparse, .. } = self;
        let dense = dense.as_mut_slice();
        let sparse = sparse.as_mut_slice();

        if first_key == second_key {
            return;
        }

        let first_index = sparse
            .get(first_key)
            .and_then(Slot::occupied)
            .map(OccupiedSlot::into_dense_index);
        let second_index = sparse
            .get(second_key)
            .and_then(Slot::occupied)
            .map(OccupiedSlot::into_dense_index);
        let (Some(first_index), Some(second_index)) = (first_index, second_index) else {
            return;
        };

        let Some((first, second)) = get_pair_mut(dense, first_index, second_index) else {
            panic!("indices from sparse should be in bounds of dense and differ from each other");
        };
        let first_value = first.value_mut();
        let second_value = second.value_mut();
        swap(first_value, second_value);
    }

    pub fn swap_remove(&mut self, key: usize) -> Option<T> {
        let Self {
            dense,
            sparse,
            first_free,
            last_free,
        } = self;

        let dense_index = sparse
            .get(key)
            .and_then(Slot::occupied)
            .map(OccupiedSlot::into_dense_index)?;
        if dense_index >= dense.len() {
            panic!("index from sparse should be in bounds of dense");
        }

        let entry = dense.swap_remove(dense_index);
        debug_assert_eq!(key, entry.key);

        if let Some(entry) = dense.get(dense_index) {
            let to_swapped = sparse
                .get_mut(entry.key)
                .expect("key from dense should point to valid sparse slot")
                .occupied_mut()
                .expect("key from dense should point to occupied sparse slot")
                .dense_index_mut();
            debug_assert_eq!(*to_swapped, entry.key);
            *to_swapped = dense_index;
        }

        // TODO free slot by O(1) complexity
        let next_free = match sparse.get_mut(*first_free) {
            Some(Slot::Free(FreeSlot { next_free, .. })) if *next_free == *first_free => {
                let result = *next_free;
                *next_free = key;
                *first_free = usize::min(key, *first_free);
                *last_free = usize::max(key, *last_free);
                result
            }
            Some(Slot::Free(_)) => {
                let (left, right) = sparse.split_at_mut(key);
                let left_free_slot = left
                    .iter_mut()
                    .enumerate()
                    .skip(*first_free)
                    .filter_map(|(idx, slot)| slot.free_mut().map(|free| (idx, free)))
                    .next_back();
                let right_free_slot = right
                    .iter_mut()
                    .enumerate()
                    .map(|(idx, slot)| (idx + key, slot))
                    .skip(1)
                    .filter_map(|(idx, slot)| slot.free_mut().map(|free| (idx, free)))
                    .next();
                match (left_free_slot, right_free_slot) {
                    (Some((_, left)), Some((right_index, _))) => {
                        left.next_free = key;
                        right_index
                    }
                    (None, Some((right_index, _))) => {
                        *first_free = key;
                        let to_first_free = sparse[*last_free]
                            .free_mut()
                            .expect("last free should point to free slot")
                            .next_free_mut();
                        *to_first_free = *first_free;
                        right_index
                    }
                    (Some((_, left)), None) => {
                        left.next_free = key;
                        *last_free = key;
                        *first_free
                    }
                    (None, None) => unreachable!("found slot should be free"),
                }
            }
            Some(Slot::Occupied { .. }) => panic!("first free should point to free slot"),
            None => {
                *first_free = key;
                *last_free = key;
                key
            }
        };
        sparse[key] = FreeSlot::new(usize::MAX, next_free).into();

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

        let dense_index = sparse
            .get(key)
            .and_then(Slot::occupied)
            .map(OccupiedSlot::into_dense_index)?;
        if dense_index >= dense.len() {
            panic!("index from sparse should be in bounds of dense");
        }

        for entry in dense.iter_mut().skip(dense_index + 1) {
            let sparse_index = entry.key;
            let occupied_slot = sparse
                .get_mut(sparse_index)
                .expect("key from dense should point to valid sparse slot")
                .occupied_mut()
                .expect("key from dense should point to occupied sparse slot");
            let OccupiedSlot { dense_index } = occupied_slot;
            *dense_index -= 1;
        }

        let entry = dense.remove(dense_index);
        debug_assert_eq!(key, entry.key);

        // TODO free slot by O(1) complexity
        let next_free = match sparse.get_mut(*first_free) {
            Some(Slot::Free(FreeSlot { next_free, .. })) if *next_free == *first_free => {
                let result = *next_free;
                *next_free = key;
                *first_free = usize::min(key, *first_free);
                *last_free = usize::max(key, *last_free);
                result
            }
            Some(Slot::Free(_)) => {
                let (left, right) = sparse.split_at_mut(key);
                let left_free_slot = left
                    .iter_mut()
                    .enumerate()
                    .skip(*first_free)
                    .filter_map(|(idx, slot)| slot.free_mut().map(|free| (idx, free)))
                    .next_back();
                let right_free_slot = right
                    .iter_mut()
                    .enumerate()
                    .map(|(idx, slot)| (idx + key, slot))
                    .skip(1)
                    .filter_map(|(idx, slot)| slot.free_mut().map(|free| (idx, free)))
                    .next();
                match (left_free_slot, right_free_slot) {
                    (Some((_, left)), Some((right_index, _))) => {
                        left.next_free = key;
                        right_index
                    }
                    (None, Some((right_index, _))) => {
                        *first_free = key;
                        let to_first_free = sparse[*last_free]
                            .free_mut()
                            .expect("last free should point to free slot")
                            .next_free_mut();
                        *to_first_free = *first_free;
                        right_index
                    }
                    (Some((_, left)), None) => {
                        left.next_free = key;
                        *last_free = key;
                        *first_free
                    }
                    (None, None) => unreachable!("found slot should be free"),
                }
            }
            Some(Slot::Occupied { .. }) => panic!("first free should point to free slot"),
            None => {
                *first_free = key;
                *last_free = key;
                key
            }
        };
        sparse[key] = FreeSlot::new(usize::MAX, next_free).into();

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        let Self { dense, sparse, .. } = self;
        let dense = dense.as_slice();
        let sparse = sparse.as_slice();

        let slot = sparse.get(key).copied()?;
        let dense_index = slot.occupied()?.into_dense_index();
        let entry = dense.get(dense_index)?;
        debug_assert_eq!(key, entry.key);

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        let Self { dense, sparse, .. } = self;
        let dense = dense.as_mut_slice();
        let sparse = sparse.as_mut_slice();

        let slot = sparse.get(key).copied()?;
        let dense_index = slot.occupied()?.into_dense_index();
        let entry = dense.get_mut(dense_index)?;
        debug_assert_eq!(key, entry.key);

        let Entry { value, .. } = entry;
        Some(value)
    }

    pub fn contains(&self, key: usize) -> bool {
        let Self { dense, sparse, .. } = self;
        let dense = dense.as_slice();
        let sparse = sparse.as_slice();

        let Some(slot) = sparse.get(key).copied() else {
            return false;
        };
        let Slot::Occupied(occupied) = slot else {
            return false;
        };

        let OccupiedSlot { dense_index } = occupied;
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

    use crate::{FreeSlot, OccupiedSlot, Slot, SparseSet};

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
        let dense = dense.as_slice();
        let sparse = sparse.as_slice();

        let mut calculated_first_free = None;
        let mut calculated_last_free = None;
        for (key, slot) in sparse.iter().copied().enumerate() {
            match slot {
                Slot::Occupied(occupied) => {
                    let OccupiedSlot { dense_index } = occupied;
                    let entry = dense
                        .get(dense_index)
                        .expect("index from sparse should be in bounds of dense");
                    assert_eq!(
                        entry.key, key,
                        "key from dense should be equal to sparse index",
                    );
                }
                Slot::Free(free) => {
                    let FreeSlot { next_free, .. } = free;
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
                    let Slot::Free(_) = sparse[next_free] else {
                        panic!("next free should point to free slot");
                    };

                    if let Some(prev_free_slot) = calculated_last_free.map(|key| sparse[key]) {
                        let Slot::Free(free) = prev_free_slot else {
                            panic!("next free should point to free slot");
                        };
                        let FreeSlot { next_free, .. } = free;
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
                let Slot::Free(FreeSlot { next_free, .. }) = sparse[calculated_last_free] else {
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
    }

    #[test]
    fn with_capacity() {
        let sparse_set = SparseSet::<i32>::with_capacity_all(10);
        debug_invariants(&sparse_set, "Empty with capacity");

        assert!(sparse_set.is_empty());
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
