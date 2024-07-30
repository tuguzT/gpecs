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

    pub fn into_boxed_slice(self) -> Box<[T]> {
        let Self { dense_values, .. } = self;
        dense_values.into_boxed_slice()
    }

    pub fn as_ptr(&self) -> *const T {
        let Self { dense_values, .. } = self;
        dense_values.as_ptr()
    }

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
            sparse.resize(key + 1, SparseEntry::Vacant);
        }

        let sparse = sparse.as_mut_slice();
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

    pub fn keys(&self) -> Keys<'_, T> {
        let Self { dense_keys, .. } = self;

        let keys = dense_keys.iter();
        let phantom = PhantomData;
        Keys { keys, phantom }
    }

    pub fn into_keys(self) -> IntoKeys<T> {
        let Self { dense_keys, .. } = self;

        let keys = dense_keys.into_iter();
        let phantom = PhantomData;
        IntoKeys { keys, phantom }
    }

    pub fn values(&self) -> Values<'_, T> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter();
        Values { values }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, T> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter_mut();
        ValuesMut { values }
    }

    pub fn into_values(self) -> IntoValues<T> {
        let Self { dense_values, .. } = self;

        let values = dense_values.into_iter();
        IntoValues { values }
    }

    pub fn iter(&self) {
        todo!()
    }

    pub fn iter_mut(&mut self) {
        todo!()
    }
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

pub struct Keys<'a, T> {
    keys: slice::Iter<'a, usize>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> Keys<'a, T> {
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
    fn default() -> Self {
        let keys = Default::default();
        let phantom = Default::default();
        Self { keys, phantom }
    }
}

impl<'a, T> Clone for Keys<'a, T> {
    fn clone(&self) -> Self {
        let Self { keys, phantom } = self;

        let keys = keys.clone();
        let phantom = *phantom;
        Self { keys, phantom }
    }
}

impl<'a, T> AsRef<[usize]> for Keys<'a, T> {
    fn as_ref(&self) -> &[usize] {
        self.as_slice()
    }
}

impl<'a, T> Iterator for Keys<'a, T> {
    type Item = &'a usize;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, .. } = self;
        keys.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.last()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.nth(n)
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { keys, .. } = self;
        keys.for_each(f)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { keys, .. } = self;
        keys.fold(init, f)
    }

    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.all(f)
    }

    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.any(f)
    }

    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.find(predicate)
    }

    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { keys, .. } = self;
        keys.find_map(f)
    }

    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.position(predicate)
    }
}

impl<'a, T> DoubleEndedIterator for Keys<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.nth_back(n)
    }
}

impl<'a, T> ExactSizeIterator for Keys<'a, T> {
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
    pub fn as_slice(&self) -> &[usize] {
        let Self { keys, .. } = self;
        keys.as_slice()
    }

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
    fn default() -> Self {
        let keys = Default::default();
        let phantom = Default::default();
        Self { keys, phantom }
    }
}

impl<T> Clone for IntoKeys<T> {
    fn clone(&self) -> Self {
        let Self { keys, phantom } = self;

        let keys = keys.clone();
        let phantom = *phantom;
        Self { keys, phantom }
    }
}

impl<T> AsRef<[usize]> for IntoKeys<T> {
    fn as_ref(&self) -> &[usize] {
        self.as_slice()
    }
}

impl<T> AsMut<[usize]> for IntoKeys<T> {
    fn as_mut(&mut self) -> &mut [usize] {
        self.as_mut_slice()
    }
}

impl<T> Iterator for IntoKeys<T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { keys, .. } = self;
        keys.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { keys, .. } = self;
        keys.last()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.nth(n)
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { keys, .. } = self;
        keys.for_each(f)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { keys, .. } = self;
        keys.fold(init, f)
    }

    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.all(f)
    }

    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.any(f)
    }

    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.find(predicate)
    }

    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { keys, .. } = self;
        keys.find_map(f)
    }

    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { keys, .. } = self;
        keys.position(predicate)
    }
}

impl<T> DoubleEndedIterator for IntoKeys<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { keys, .. } = self;
        keys.nth_back(n)
    }
}

impl<T> ExactSizeIterator for IntoKeys<T> {
    fn len(&self) -> usize {
        let Self { keys, .. } = self;
        keys.len()
    }
}

impl<T> FusedIterator for IntoKeys<T> {}

#[derive(Default, Clone)]
pub struct Values<'a, T> {
    values: slice::Iter<'a, T>,
}

impl<'a, T> Values<'a, T> {
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

impl<'a, T> AsRef<[T]> for Values<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { values } = self;
        values.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.last()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth(n)
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { values } = self;
        values.for_each(f)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { values } = self;
        values.fold(init, f)
    }

    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.all(f)
    }

    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.any(f)
    }

    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { values } = self;
        values.find(predicate)
    }

    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { values } = self;
        values.find_map(f)
    }

    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.position(predicate)
    }
}

impl<'a, T> DoubleEndedIterator for Values<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth_back(n)
    }
}

impl<'a, T> ExactSizeIterator for Values<'a, T> {
    fn len(&self) -> usize {
        let Self { values } = self;
        values.len()
    }
}

impl<'a, T> FusedIterator for Values<'a, T> {}

#[derive(Default)]
pub struct ValuesMut<'a, T> {
    values: slice::IterMut<'a, T>,
}

impl<'a, T> ValuesMut<'a, T> {
    pub fn into_slice(self) -> &'a [T] {
        let Self { values } = self;
        values.into_slice()
    }

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

impl<'a, T> AsRef<[T]> for ValuesMut<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { values } = self;
        values.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.last()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth(n)
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { values } = self;
        values.for_each(f)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { values } = self;
        values.fold(init, f)
    }

    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.all(f)
    }

    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.any(f)
    }

    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { values } = self;
        values.find(predicate)
    }

    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { values } = self;
        values.find_map(f)
    }

    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.position(predicate)
    }
}

impl<'a, T> DoubleEndedIterator for ValuesMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth_back(n)
    }
}

impl<'a, T> ExactSizeIterator for ValuesMut<'a, T> {
    fn len(&self) -> usize {
        let Self { values } = self;
        values.len()
    }
}

impl<'a, T> FusedIterator for ValuesMut<'a, T> {}

#[derive(Default, Clone)]
pub struct IntoValues<T> {
    values: vec::IntoIter<T>,
}

impl<T> IntoValues<T> {
    pub fn as_slice(&self) -> &[T] {
        let Self { values } = self;
        values.as_slice()
    }

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

impl<T> AsRef<[T]> for IntoValues<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for IntoValues<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> Iterator for IntoValues<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { values } = self;
        values.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { values } = self;
        values.last()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth(n)
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        let Self { values } = self;
        values.for_each(f)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { values } = self;
        values.fold(init, f)
    }

    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.all(f)
    }

    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.any(f)
    }

    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { values } = self;
        values.find(predicate)
    }

    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { values } = self;
        values.find_map(f)
    }

    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let Self { values } = self;
        values.position(predicate)
    }
}

impl<T> DoubleEndedIterator for IntoValues<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { values } = self;
        values.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { values } = self;
        values.nth_back(n)
    }
}

impl<T> ExactSizeIterator for IntoValues<T> {
    fn len(&self) -> usize {
        let Self { values } = self;
        values.len()
    }
}

impl<T> FusedIterator for IntoValues<T> {}

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
        let mut sparse_set = SparseSet::with_capacity_all(10);
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
        let mut sparse_set = SparseSet::with_capacity_all(10);
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
}
