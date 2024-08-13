use alloc::{
    boxed::Box,
    collections::TryReserveError,
    vec::{self, Vec},
};
use core::{
    cmp,
    fmt::{self, Debug, Display},
    iter::FusedIterator,
    marker::PhantomData,
    mem::{replace, swap},
    ops::{Index, IndexMut},
    slice,
};

use crate::{
    check_dense_index_bounds, check_equal_key, check_key_bounds, check_kv_same_capacity,
    check_kv_same_len, get_pair_mut,
    key::{Epoch, Key},
    match_kv_same_kind, unwrap_dense_index, unwrap_dense_index_mut, unwrap_dense_key,
    unwrap_dense_value, unwrap_dense_value_mut, unwrap_dense_value_pair_mut, unwrap_sparse_item,
    unwrap_sparse_item_mut, unwrap_sparse_items_pair_mut, unwrap_value_from_sparse_index,
    SparseItem, SparseItemKind,
};

pub type SparseSet<T> = EpochSparseSet<usize, T>;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EpochSparseSet<K, V>
where
    K: Key,
{
    dense_keys: Vec<K>,
    dense_values: Vec<V>,
    sparse: Vec<SparseItem<K::Epoch>>,
}

impl<K, V> EpochSparseSet<K, V>
where
    K: Key,
{
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

        check_kv_same_len(dense_keys.len(), dense_values.len());
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

        check_kv_same_capacity(dense_keys.capacity(), dense_values.capacity());
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
    pub fn as_slice(&self) -> &[V] {
        let Self { dense_values, .. } = self;
        dense_values.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [V] {
        let Self { dense_values, .. } = self;
        dense_values.as_mut_slice()
    }

    #[inline]
    pub fn into_boxed_slice(self) -> Box<[V]> {
        let Self { dense_values, .. } = self;
        dense_values.into_boxed_slice()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const V {
        let Self { dense_values, .. } = self;
        dense_values.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut V {
        let Self { dense_values, .. } = self;
        dense_values.as_mut_ptr()
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { dense_keys, .. } = self;
        dense_keys.as_slice()
    }

    #[inline]
    pub fn into_keys_boxed_slice(self) -> Box<[K]> {
        let Self { dense_keys, .. } = self;
        dense_keys.into_boxed_slice()
    }

    #[inline]
    pub fn as_keys_ptr(&self) -> *const K {
        let Self { dense_keys, .. } = self;
        dense_keys.as_ptr()
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &[SparseItem<K::Epoch>] {
        let Self { sparse, .. } = self;
        sparse.as_slice()
    }

    #[inline]
    pub fn into_sparse_boxed_slice(self) -> Box<[SparseItem<K::Epoch>]> {
        let Self { sparse, .. } = self;
        sparse.into_boxed_slice()
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const SparseItem<K::Epoch> {
        let Self { sparse, .. } = self;
        sparse.as_ptr()
    }

    #[inline]
    pub fn into_parts(self) -> (Vec<K>, Vec<V>, Vec<SparseItem<K::Epoch>>) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        (dense_keys, dense_values, sparse)
    }

    pub fn from_parts(
        mut keys: Vec<K>,
        mut values: Vec<V>,
        mut sparse: Vec<SparseItem<K::Epoch>>,
    ) -> Self {
        keys.dedup_by_key(|key| key.sparse_index());
        values.truncate(keys.len());
        keys.truncate(values.len());
        check_kv_same_len(keys.len(), values.len());

        sparse.clear();
        for (dense_index, key) in keys.iter().enumerate() {
            let sparse_index = key.sparse_index();
            let epoch = key.epoch();
            let item = SparseItem::occupied(dense_index, epoch);

            if sparse_index >= sparse.len() {
                let epoch = Default::default();
                let item = SparseItem::vacant(0, epoch);
                sparse.resize(sparse_index.saturating_add(1), item);
            }
            sparse[sparse_index] = item;
        }

        Self {
            dense_keys: keys,
            dense_values: values,
            sparse,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_index = key.sparse_index();
        if sparse_index >= sparse.len() {
            let epoch = Default::default();
            let item = SparseItem::vacant(0, epoch);
            sparse.resize(sparse_index.saturating_add(1), item);
        }

        let sparse_item = sparse.index_mut(sparse_index);
        if key.epoch() < sparse_item.epoch {
            return None;
        }

        if let SparseItemKind::Occupied { dense_index } = sparse_item.kind {
            let value_mut = unwrap_dense_value_mut(dense_values, dense_index);
            let value = replace(value_mut, value);
            sparse_item.epoch = key.epoch();
            dense_keys[dense_index] = key;
            return Some(value);
        }

        check_kv_same_len(dense_keys.len(), dense_values.len());
        dense_keys.push(key);
        dense_values.push(value);
        *sparse_item = SparseItem::occupied(dense_keys.len() - 1, key.epoch());

        None
    }

    pub fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_index = key.sparse_index();
        if sparse_index >= sparse.len() {
            let new_sparse_len = sparse_index.saturating_add(1);
            sparse.try_reserve(new_sparse_len - sparse.len())?;

            let epoch = Default::default();
            let item = SparseItem::vacant(0, epoch);
            sparse.resize(new_sparse_len, item);
        }

        let sparse_item = sparse.index_mut(sparse_index);
        if key.epoch() < sparse_item.epoch {
            return Ok(None);
        }

        if let SparseItemKind::Occupied { dense_index } = sparse_item.kind {
            let value_mut = unwrap_dense_value_mut(dense_values, dense_index);
            let value = replace(value_mut, value);
            sparse_item.epoch = key.epoch();
            dense_keys[dense_index] = key;
            return Ok(Some(value));
        }

        check_kv_same_len(dense_keys.len(), dense_values.len());
        dense_keys.try_reserve(1)?;
        dense_values.try_reserve(1)?;

        dense_keys.push(key);
        dense_values.push(value);
        *sparse_item = SparseItem::occupied(dense_keys.len() - 1, key.epoch());

        Ok(None)
    }

    pub fn push(&mut self, value: V) -> K {
        let Self { sparse, .. } = self;

        let (sparse_index, epoch) = sparse
            .iter()
            .enumerate()
            .find(|(_, item)| item.is_vacant())
            .map(|(sparse_index, item)| (sparse_index, item.epoch))
            .unwrap_or((self.sparse.len(), Default::default()));
        let key = K::new(sparse_index, epoch);

        self.insert(key, value);
        key
    }

    pub fn try_push(&mut self, value: V) -> Result<K, TryReserveError> {
        let Self { sparse, .. } = self;

        let (sparse_index, epoch) = sparse
            .iter()
            .enumerate()
            .find(|(_, item)| item.is_vacant())
            .map(|(sparse_index, item)| (sparse_index, item.epoch))
            .unwrap_or((self.sparse.len(), Default::default()));
        let key = K::new(sparse_index, epoch);

        self.try_insert(key, value)?;
        Ok(key)
    }

    pub fn swap(&mut self, first_key: K, second_key: K) {
        let Self {
            dense_values,
            sparse,
            ..
        } = self;

        let first_index = first_key.sparse_index();
        let second_index = second_key.sparse_index();
        if first_index == second_index {
            return;
        }

        let Some(first_index) = sparse
            .get(first_index)
            .take_if(|item| item.epoch == first_key.epoch())
            .and_then(SparseItem::dense_index)
        else {
            return;
        };
        let Some(second_index) = sparse
            .get(second_index)
            .take_if(|item| item.epoch == second_key.epoch())
            .and_then(SparseItem::dense_index)
        else {
            return;
        };

        let (first_value, second_value) =
            unwrap_dense_value_pair_mut(dense_values, first_index, second_index);
        swap(first_value, second_value);
    }

    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let Self {
            dense_keys, sparse, ..
        } = self;

        let first_index = first_key.sparse_index();
        let second_index = second_key.sparse_index();
        let Some((first_item, second_item)) = get_pair_mut(sparse, first_index, second_index)
        else {
            return;
        };

        let Some(first_index) = Some(&*first_item)
            .take_if(|item| item.epoch == first_key.epoch())
            .and_then(SparseItem::dense_index)
        else {
            return;
        };
        let Some(second_index) = Some(&*second_item)
            .take_if(|item| item.epoch == second_key.epoch())
            .and_then(SparseItem::dense_index)
        else {
            return;
        };

        let (first_key, second_key) =
            unwrap_dense_value_pair_mut(dense_keys, first_index, second_index);
        swap(first_item, second_item);
        swap(first_key, second_key);
    }

    pub fn swap_remove(&mut self, key: K) -> Option<V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_index = key.sparse_index();
        let dense_index = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)?;
        check_dense_index_bounds(dense_index, dense_keys.len());

        check_kv_same_len(dense_keys.len(), dense_values.len());
        let value = dense_values.swap_remove(dense_index);
        let dense_key = dense_keys.swap_remove(dense_index);
        check_equal_key(key, dense_key);

        if let Some(swapped_key) = dense_keys.get(dense_index) {
            let sparse_index = swapped_key.sparse_index();
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            if let Some(swapped_dense_index) = sparse_item.dense_index_mut() {
                *swapped_dense_index = dense_index;
            }
        }
        sparse[sparse_index] = SparseItem::vacant(0, key.epoch().next());

        Some(value)
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_index = key.sparse_index();
        let dense_index = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)?;
        check_dense_index_bounds(dense_index, dense_keys.len());

        check_kv_same_len(dense_keys.len(), dense_values.len());
        let value = dense_values.remove(dense_index);
        let dense_key = dense_keys.remove(dense_index);
        check_equal_key(key, dense_key);

        for key in dense_keys.iter().copied().skip(dense_index) {
            let sparse_index = key.sparse_index();
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            let dense_index = unwrap_dense_index_mut(sparse_item.kind_mut());
            *dense_index -= 1;
        }
        sparse[sparse_index] = SparseItem::vacant(0, key.epoch().next());

        Some(value)
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let key = dense_keys.pop();
        let value = dense_values.pop();
        let (key, value) = match_kv_same_kind(key, value)?;

        let sparse_index = key.sparse_index();
        check_key_bounds(sparse_index, sparse.len());
        sparse[sparse_index] = SparseItem::vacant(0, key.epoch().next());

        Some((key, value))
    }

    pub fn truncate(&mut self, dense_len: usize, sparse_len: usize) {
        for dense_index in (dense_len..self.len()).rev() {
            let key = self.dense_keys[dense_index];
            self.remove(key);
        }
        self.dense_keys.truncate(dense_len);
        self.dense_values.truncate(dense_len);

        for sparse_index in sparse_len..self.sparse_len() {
            let epoch = self.sparse[sparse_index].epoch;
            let key = K::new(sparse_index, epoch.next());
            self.remove(key);
        }
        self.sparse.truncate(sparse_len);
    }

    pub fn drain(&mut self) -> Drain<'_, K, V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let keys = dense_keys.drain(..);
        let values = dense_values.drain(..);
        sparse.clear();

        Drain::new(keys, values)
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(K, &mut V) -> bool,
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
        V: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                unwrap_value_from_sparse_index(sparse_index, values, sparse)
            })
        });
    }

    #[inline]
    pub fn sort_keys(&mut self) {
        self.sort_impl(|keys, _, _| keys.sort());
    }

    #[inline]
    pub fn sort_by<F>(&mut self, mut f: F)
    where
        F: FnMut((K, &V), (K, &V)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by(|&lhs_key, &rhs_key| {
                let lhs_index = lhs_key.sparse_index();
                let lhs_value = unwrap_value_from_sparse_index(lhs_index, values, sparse);
                let lhs = (lhs_key, lhs_value);

                let rhs_index = rhs_key.sparse_index();
                let rhs_value = unwrap_value_from_sparse_index(rhs_index, values, sparse);
                let rhs = (rhs_key, rhs_value);

                f(lhs, rhs)
            })
        });
    }

    #[inline]
    pub fn sort_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, &V)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let value = unwrap_value_from_sparse_index(sparse_index, values, sparse);
                f((key, value))
            })
        });
    }

    #[inline]
    pub fn sort_by_cached_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, &V)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                let value = unwrap_value_from_sparse_index(sparse_index, values, sparse);
                f((key, value))
            })
        });
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        V: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                unwrap_value_from_sparse_index(sparse_index, values, sparse)
            })
        });
    }

    #[inline]
    pub fn sort_keys_unstable(&mut self) {
        self.sort_impl(|keys, _, _| keys.sort_unstable());
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, mut f: F)
    where
        F: FnMut((K, &V), (K, &V)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by(|&lhs_key, &rhs_key| {
                let lhs_index = lhs_key.sparse_index();
                let lhs_value = unwrap_value_from_sparse_index(lhs_index, values, sparse);
                let lhs = (lhs_key, lhs_value);

                let rhs_index = rhs_key.sparse_index();
                let rhs_value = unwrap_value_from_sparse_index(rhs_index, values, sparse);
                let rhs = (rhs_key, rhs_value);

                f(lhs, rhs)
            })
        });
    }

    #[inline]
    pub fn sort_unstable_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, &V)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let value = unwrap_value_from_sparse_index(sparse_index, values, sparse);
                f((key, value))
            })
        });
    }

    // Implementation was borrowed from the links below:
    // https://skypjack.github.io/2019-09-25-ecs-baf-part-5/#:~:text=Mixing%20in%2Dplace%20sorting%20and%20permutations
    // https://github.com/skypjack/entt/blob/8b0ef2b94234def2053c9a8a2591f4a5e87cf0ea/src/entt/entity/sparse_set.hpp#L964
    fn sort_impl<SortKeys>(&mut self, sort_keys: SortKeys)
    where
        SortKeys: FnOnce(&mut [K], &[V], &[SparseItem<K::Epoch>]),
    {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let dense_keys = dense_keys.as_mut_slice();
        let dense_values = dense_values.as_mut_slice();
        let sparse = sparse.as_mut_slice();
        check_kv_same_len(dense_keys.len(), dense_values.len());

        sort_keys(dense_keys, dense_values, sparse);

        for pos in 0..dense_keys.len() {
            let mut curr = pos;
            let mut next = {
                let sparse_index = unwrap_dense_key(dense_keys, curr).sparse_index();
                let sparse_item = unwrap_sparse_item(sparse, sparse_index);
                unwrap_dense_index(sparse_item.kind())
            };

            while curr != next {
                let (curr_item, next_item) = {
                    let first_index = unwrap_dense_key(dense_keys, curr).sparse_index();
                    let second_index = unwrap_dense_key(dense_keys, next).sparse_index();
                    unwrap_sparse_items_pair_mut(sparse, first_index, second_index)
                };
                let curr_dense_index = unwrap_dense_index_mut(curr_item.kind_mut());
                let next_dense_index = unwrap_dense_index_mut(next_item.kind_mut());

                dense_values.swap(*curr_dense_index, *next_dense_index);

                *curr_dense_index = curr;
                curr = next;
                next = *next_dense_index;
            }
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_index = key.sparse_index();
        let sparse_item = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())?;
        let dense_index = sparse_item.dense_index()?;

        let value = unwrap_dense_value(dense_values, dense_index);
        let dense_key = unwrap_dense_key(dense_keys, dense_index);
        check_equal_key(key, *dense_key);

        Some(value)
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_index = key.sparse_index();
        let sparse_item = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())?;
        let dense_index = sparse_item.dense_index()?;

        let value = unwrap_dense_value_mut(dense_values, dense_index);
        let dense_key = unwrap_dense_key(dense_keys, dense_index);
        check_equal_key(key, *dense_key);

        Some(value)
    }

    pub fn get_with_key(&self, sparse_index: usize) -> Option<(K, &V)> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_item = sparse.get(sparse_index)?;
        let dense_index = sparse_item.dense_index()?;

        let value = unwrap_dense_value(dense_values, dense_index);
        let key = *unwrap_dense_key(dense_keys, dense_index);
        check_equal_key(key, K::new(sparse_index, sparse_item.epoch));

        Some((key, value))
    }

    pub fn get_mut_with_key(&mut self, sparse_index: usize) -> Option<(K, &mut V)> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        let sparse_item = sparse.get(sparse_index)?;
        let dense_index = sparse_item.dense_index()?;

        let value = unwrap_dense_value_mut(dense_values, dense_index);
        let key = *unwrap_dense_key(dense_keys, dense_index);
        check_equal_key(key, K::new(sparse_index, sparse_item.epoch));

        Some((key, value))
    }

    pub fn get_epoch(&self, sparse_index: usize) -> Option<K::Epoch> {
        let Self {
            sparse, dense_keys, ..
        } = self;

        let sparse_item = sparse.get(sparse_index)?;
        let epoch = sparse_item.epoch;
        if let Some(dense_index) = sparse_item.dense_index() {
            let key = *unwrap_dense_key(dense_keys, dense_index);
            check_equal_key(key, K::new(sparse_index, epoch));
        }

        Some(epoch)
    }

    pub fn contains_key(&self, key: K) -> bool {
        let Self {
            dense_keys, sparse, ..
        } = self;

        let sparse_index = key.sparse_index();
        let Some(sparse_item) = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .copied()
        else {
            return false;
        };
        let SparseItemKind::Occupied { dense_index } = sparse_item.kind else {
            return false;
        };

        check_dense_index_bounds(dense_index, dense_keys.len());
        true
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        let Self {
            dense_keys, sparse, ..
        } = self;

        let sparse_index = key.sparse_index();
        let Some(dense_index) = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)
        else {
            let sparse_set = self;
            let entry = VacantEntry { key, sparse_set };
            return Entry::Vacant(entry);
        };

        check_dense_index_bounds(dense_index, dense_keys.len());
        let entry = OccupiedEntry {
            key,
            dense_index,
            sparse_set: self,
        };
        Entry::Occupied(entry)
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

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        let Self { dense_keys, .. } = self;

        let keys = dense_keys.iter();
        Keys::new(keys)
    }

    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        let Self { dense_keys, .. } = self;

        let keys = dense_keys.into_iter();
        IntoKeys::new(keys)
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter();
        Values::new(values)
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter_mut();
        ValuesMut::new(values)
    }

    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        let Self { dense_values, .. } = self;

        let values = dense_values.into_iter();
        IntoValues::new(values)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.iter();
        let values = dense_values.iter();
        Iter::new(keys, values)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.iter();
        let values = dense_values.iter_mut();
        IterMut::new(keys, values)
    }
}

impl<K, V> Index<K> for EpochSparseSet<K, V>
where
    K: Key + Display,
{
    type Output = V;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        match self.get(key) {
            Some(value) => value,
            None => panic!("key {key} not found"),
        }
    }
}

impl<K, V> IndexMut<K> for EpochSparseSet<K, V>
where
    K: Key + Display,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        match self.get_mut(key) {
            Some(value) => value,
            None => panic!("key {key} not found"),
        }
    }
}

impl<K, V> AsRef<[V]> for EpochSparseSet<K, V>
where
    K: Key,
{
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<K, V> AsMut<[V]> for EpochSparseSet<K, V>
where
    K: Key,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [V] {
        self.as_mut_slice()
    }
}

impl<K, V> AsRef<EpochSparseSet<K, V>> for EpochSparseSet<K, V>
where
    K: Key,
{
    #[inline]
    fn as_ref(&self) -> &EpochSparseSet<K, V> {
        self
    }
}

impl<K, V> AsMut<EpochSparseSet<K, V>> for EpochSparseSet<K, V>
where
    K: Key,
{
    #[inline]
    fn as_mut(&mut self) -> &mut EpochSparseSet<K, V> {
        self
    }
}

impl<'a, K, V> IntoIterator for &'a EpochSparseSet<K, V>
where
    K: Key,
{
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut EpochSparseSet<K, V>
where
    K: Key,
{
    type Item = (&'a K, &'a mut V);

    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V> IntoIterator for EpochSparseSet<K, V>
where
    K: Key,
{
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.into_iter();
        let values = dense_values.into_iter();
        IntoIter::new(keys, values)
    }
}

impl<K, V> FromIterator<(K, V)> for EpochSparseSet<K, V>
where
    K: Key,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
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

impl<K, V> FromIterator<V> for EpochSparseSet<K, V>
where
    K: Key,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        let dense_values: Vec<_> = iter.into_iter().collect();

        let len = dense_values.len();
        let dense_keys = (0..len)
            .map(|sparse_index| K::new(sparse_index, Default::default()))
            .collect();
        let sparse = (0..len)
            .map(|dense_index| SparseItem::occupied(dense_index, Default::default()))
            .collect();

        Self {
            dense_keys,
            dense_values,
            sparse,
        }
    }
}

impl<K, V> Extend<(K, V)> for EpochSparseSet<K, V>
where
    K: Key,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
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

impl<K, V> Extend<V> for EpochSparseSet<K, V>
where
    K: Key,
{
    fn extend<I: IntoIterator<Item = V>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let iter_len = {
            let (lower, upper) = iter.size_hint();
            upper.unwrap_or(lower)
        };
        self.reserve(iter_len, iter_len);

        let mut maybe_vacant_keys = 0..self.sparse.len();
        for value in iter {
            let sparse_index = maybe_vacant_keys
                .find(|&key| self.sparse[key].is_vacant())
                .unwrap_or(self.sparse.len());
            let key = K::new(sparse_index, Default::default());
            self.insert(key, value);
        }
    }
}

#[derive(Debug)]
pub enum Entry<'a, K, V>
where
    K: Key,
{
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Key,
{
    #[inline]
    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied(_))
    }

    #[inline]
    pub const fn is_vacant(&self) -> bool {
        matches!(self, Self::Vacant(_))
    }

    #[inline]
    pub fn key(&self) -> K {
        match self {
            Self::Occupied(entry) => entry.key(),
            Self::Vacant(entry) => entry.key(),
        }
    }

    #[inline]
    pub fn get(&self) -> Option<&V> {
        match self {
            Self::Occupied(entry) => Some(entry.get()),
            Self::Vacant(_) => None,
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut V> {
        match self {
            Self::Occupied(entry) => Some(entry.get_mut()),
            Self::Vacant(_) => None,
        }
    }

    #[inline]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Self::Occupied(mut entry) => {
                f(entry.get_mut());
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => Self::Vacant(entry),
        }
    }

    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default),
        }
    }

    #[inline]
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default()),
        }
    }

    #[inline]
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(Default::default()),
        }
    }

    #[inline]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V> {
        match self {
            Self::Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
            Self::Vacant(entry) => entry.insert_entry(value),
        }
    }

    #[inline]
    pub fn replace_key(self, key: K) -> Self {
        match self {
            Self::Occupied(mut entry) => {
                entry.replace_key(key);
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => {
                let VacantEntry { sparse_set, .. } = entry;
                sparse_set.entry(key)
            }
        }
    }
}

pub struct OccupiedEntry<'a, K, V>
where
    K: Key,
{
    key: K,
    dense_index: usize,
    sparse_set: &'a mut EpochSparseSet<K, V>,
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Key,
{
    #[inline]
    pub fn get(&self) -> &V {
        let Self {
            dense_index,
            sparse_set,
            ..
        } = self;

        let values = sparse_set.dense_values.as_slice();
        unwrap_dense_value(values, *dense_index)
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        let Self {
            dense_index,
            sparse_set,
            ..
        } = self;

        let values = sparse_set.dense_values.as_mut_slice();
        unwrap_dense_value_mut(values, *dense_index)
    }

    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        let Self {
            dense_index,
            sparse_set,
            ..
        } = self;

        let values = sparse_set.dense_values.as_mut_slice();
        unwrap_dense_value_mut(values, dense_index)
    }

    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        let previous = self.get_mut();
        replace(previous, value)
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Key,
{
    #[inline]
    pub fn key(&self) -> K {
        let Self { key, .. } = self;
        *key
    }

    #[inline]
    pub fn remove(self) -> V {
        let Self {
            key, sparse_set, ..
        } = self;

        let value = sparse_set.remove(key);
        unwrap_sparse_value(value)
    }

    #[inline]
    pub fn swap_remove(self) -> V {
        let Self {
            key, sparse_set, ..
        } = self;

        let value = sparse_set.swap_remove(key);
        unwrap_sparse_value(value)
    }

    #[inline]
    pub fn replace_key(&mut self, key: K) -> Option<V> {
        let new_key = key;
        let Self {
            key, sparse_set, ..
        } = self;

        let value = sparse_set.remove(*key);
        let value = unwrap_sparse_value(value);

        *key = new_key;
        sparse_set.insert(*key, value)
    }
}

impl<'a, K, V> Debug for OccupiedEntry<'a, K, V>
where
    K: Key + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, .. } = self;

        let value = self.get();
        f.debug_struct("OccupiedEntry")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

pub struct VacantEntry<'a, K, V>
where
    K: Key,
{
    key: K,
    sparse_set: &'a mut EpochSparseSet<K, V>,
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Key,
{
    #[inline]
    pub fn key(&self) -> K {
        let Self { key, .. } = self;
        *key
    }

    #[inline]
    pub fn insert(self, value: V) -> &'a mut V {
        let Self { key, sparse_set } = self;

        sparse_set.insert(key, value);

        let value = sparse_set.dense_values.last_mut();
        unwrap_sparse_value(value)
    }

    #[inline]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V> {
        let Self { key, sparse_set } = self;

        sparse_set.insert(key, value);
        let dense_index = sparse_set.dense_values.len() - 1;

        OccupiedEntry {
            key,
            dense_index,
            sparse_set,
        }
    }
}

impl<'a, K, V> Debug for VacantEntry<'a, K, V>
where
    K: Key + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, .. } = self;
        f.debug_struct("VacantEntry").field("key", key).finish()
    }
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_sparse_value_failed() -> ! {
    panic!("value by provided key should exist")
}

#[inline]
#[track_caller]
fn unwrap_sparse_value<T>(value: Option<T>) -> T {
    let Some(value) = value else {
        unwrap_sparse_value_failed()
    };
    value
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

#[cfg(test)]
mod tests {
    use std::{mem::forget, ops::Not};

    use crate::key::{Epoch, EpochKey, Key as _};

    use super::{EpochSparseSet, SparseItem, SparseSet};

    type Key = EpochKey<usize>;

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
    fn empty_parts() {
        let sparse_set = SparseSet::<i32>::new();

        let (keys, values, sparse) = sparse_set.into_parts();
        assert_eq!(keys.len(), 0);
        assert_eq!(values.len(), 0);
        assert_eq!(sparse.len(), 0);

        let sparse_set = SparseSet::from_parts(keys, values, sparse);
        assert_eq!(sparse_set.len(), 0);
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
    fn one_item_insert_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(0), None);
        assert!(sparse_set.contains_key(0).not());
    }

    #[test]
    fn one_item_insert_remove_one_epoch() {
        let mut sparse_set = EpochSparseSet::new();

        let key = Key::new(0, 1);
        sparse_set.insert(key, 42);

        let removed = sparse_set.remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        assert_eq!(
            sparse_set.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_insert_swap_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(0), None);
        assert!(sparse_set.contains_key(0).not());
    }

    #[test]
    fn one_item_insert_swap_remove_one_epoch() {
        let mut sparse_set = EpochSparseSet::new();

        let key = Key::new(0, 1);
        sparse_set.insert(key, 42);

        let removed = sparse_set.swap_remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        assert_eq!(
            sparse_set.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_push_remove_one() {
        let mut sparse_set = SparseSet::new();
        let key = sparse_set.push(42);

        let removed = sparse_set.remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_push_remove_one_epoch() {
        let mut sparse_set = EpochSparseSet::<Key, _>::new();
        let key = sparse_set.push(42);

        let removed = sparse_set.remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        assert_eq!(
            sparse_set.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_push_swap_remove_one() {
        let mut sparse_set = SparseSet::new();
        let key = sparse_set.push(42);

        let removed = sparse_set.swap_remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_push_swap_remove_one_epoch() {
        let mut sparse_set = EpochSparseSet::<Key, _>::new();
        let key = sparse_set.push(42);

        let removed = sparse_set.swap_remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        assert_eq!(
            sparse_set.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_swap() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        sparse_set.swap(0, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slice(), &[42]);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains_key(0));

        sparse_set.swap(0, 1);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slice(), &[42]);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn one_item_swap_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        sparse_set.swap_keys(0, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slice(), &[42]);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains_key(0));

        sparse_set.swap_keys(0, 1);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slice(), &[42]);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn one_item_parts() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, 42);

        let (keys, values, sparse) = sparse_set.into_parts();
        assert_eq!(keys, &[2]);
        assert_eq!(values, &[42]);
        assert_eq!(
            sparse,
            &[
                SparseItem::vacant(0, ()),
                SparseItem::vacant(0, ()),
                SparseItem::occupied(0, ()),
            ]
        );

        let sparse_set = SparseSet::from_parts(keys, values, sparse);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slice(), &[42]);
        assert_eq!(sparse_set.as_keys_slice(), &[2]);
        assert_eq!(sparse_set.get(2), Some(&42));
    }

    #[test]
    fn one_item_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let keys = sparse_set.keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.as_slice(), &[0]);
    }

    #[test]
    fn one_item_into_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let keys = sparse_set.into_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.as_slice(), &[0]);
    }

    #[test]
    fn one_item_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let values = sparse_set.values();
        assert_eq!(values.len(), 1);
        assert_eq!(values.as_slice(), &[42]);
    }

    #[test]
    fn one_item_values_mut() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let values_mut = sparse_set.values_mut();
        assert_eq!(values_mut.len(), 1);
        assert_eq!(values_mut.into_slice(), &mut [42]);
    }

    #[test]
    fn one_item_into_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let values = sparse_set.into_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values.as_slice(), &[42]);
    }

    #[test]
    fn one_item_iter() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let iter = sparse_set.iter();
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.as_keys_slice(), &[0]);
        assert_eq!(iter.as_values_slice(), &[42]);
    }

    #[test]
    fn one_item_iter_mut() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);

        let iter_mut = sparse_set.iter_mut();
        assert_eq!(iter_mut.len(), 1);
        assert_eq!(iter_mut.as_keys_slice(), &[0]);
        assert_eq!(iter_mut.into_values_slice(), &mut [42]);
    }

    #[test]
    fn one_item_into_iter() {
        let mut sparse_set = SparseSet::new();
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
    fn two_items_insert_first_epoch() {
        let mut sparse_set = EpochSparseSet::new();

        let first_key = Key::new(0, 3);
        sparse_set.insert(first_key, 42);

        let second_key = Key::new(1, 0);
        sparse_set.insert(second_key, 69);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(first_key), Some(&42));
        assert_eq!(sparse_set.get(second_key), Some(&69));

        let first_key = Key::new(first_key.sparse_index(), first_key.epoch().next());
        let previous = sparse_set.insert(first_key, 34);
        assert_eq!(previous, Some(42));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(first_key), Some(&34));
        assert_eq!(sparse_set.get(second_key), Some(&69));
        assert!(sparse_set.contains_key(first_key));
        assert!(sparse_set.contains_key(second_key));
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
        assert_eq!(sparse_set.as_slice(), &[42, 69]);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        sparse_set.swap(0, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slice(), &[69, 42]);
        assert_eq!(sparse_set.get(0), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&42));

        sparse_set.swap(1, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slice(), &[69, 42]);
        assert_eq!(sparse_set.get(0), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&42));
    }

    #[test]
    fn two_items_swap_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, 42);
        sparse_set.insert(1, 69);

        sparse_set.swap_keys(0, 0);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slice(), &[42, 69]);
        assert_eq!(sparse_set.get(0), Some(&42));
        assert_eq!(sparse_set.get(1), Some(&69));

        sparse_set.swap_keys(0, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slice(), &[42, 69]);
        assert_eq!(sparse_set.get(0), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&42));

        sparse_set.swap_keys(1, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slice(), &[42, 69]);
        assert_eq!(sparse_set.get(0), Some(&69));
        assert_eq!(sparse_set.get(1), Some(&42));
    }

    #[test]
    fn two_items_insert_pop() {
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
    fn two_items_push_pop() {
        let mut sparse_set = SparseSet::new();
        let first_key = sparse_set.push(42);
        let second_key = sparse_set.push(69);

        let popped = sparse_set.pop();
        assert_eq!(popped, Some((second_key, 69)));
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(first_key), Some(&42));
        assert_eq!(sparse_set.get(second_key), None);
    }

    #[test]
    fn two_items_insert_pop_epoch() {
        let mut sparse_set = EpochSparseSet::new();

        let first_key = Key::new(5, 1);
        sparse_set.insert(first_key, 42);

        let second_key = Key::new(2, 0);
        sparse_set.insert(second_key, 69);

        let popped = sparse_set.pop();
        assert_eq!(popped, Some((second_key, 69)));
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(first_key), Some(&42));
        assert_eq!(sparse_set.get(second_key), None);

        assert_eq!(
            sparse_set.get_epoch(second_key.sparse_index()),
            Some(second_key.epoch().next()),
        );
    }

    #[test]
    fn two_items_push_pop_epoch() {
        let mut sparse_set = EpochSparseSet::<Key, _>::new();
        let first_key = sparse_set.push(42);
        let second_key = sparse_set.push(69);

        let popped = sparse_set.pop();
        assert_eq!(popped, Some((second_key, 69)));
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(first_key), Some(&42));
        assert_eq!(sparse_set.get(second_key), None);

        assert_eq!(
            sparse_set.get_epoch(second_key.sparse_index()),
            Some(second_key.epoch().next()),
        );
    }

    #[test]
    fn three_items_insert_remove_middle() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let removed = sparse_set.remove(2);
        assert_eq!(removed, Some(34));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(2), None);
        assert_eq!(sparse_set.get(1), Some(&42));
        assert_eq!(sparse_set.get(5), Some(&69));
        assert!(sparse_set.contains_key(2).not());
        assert!(sparse_set.contains_key(1));
        assert!(sparse_set.contains_key(5));
    }

    #[test]
    fn three_items_push_remove_middle() {
        let mut sparse_set = SparseSet::new();
        let first_key = sparse_set.push(34);
        let middle_key = sparse_set.push(42);
        let last_key = sparse_set.push(69);

        let removed = sparse_set.remove(middle_key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(first_key), Some(&34));
        assert_eq!(sparse_set.get(middle_key), None);
        assert_eq!(sparse_set.get(last_key), Some(&69));
        assert!(sparse_set.contains_key(first_key));
        assert!(sparse_set.contains_key(middle_key).not());
        assert!(sparse_set.contains_key(last_key));
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
    fn three_items_parts() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(5, 69);

        let (mut keys, values, sparse) = sparse_set.into_parts();
        assert_eq!(keys, &[2, 1, 5]);
        assert_eq!(values, &[34, 42, 69]);
        assert_eq!(
            sparse,
            &[
                SparseItem::vacant(0, ()),
                SparseItem::occupied(1, ()),
                SparseItem::occupied(0, ()),
                SparseItem::vacant(0, ()),
                SparseItem::vacant(0, ()),
                SparseItem::occupied(2, ()),
            ]
        );

        keys.swap_remove(0);
        let sparse_set = SparseSet::from_parts(keys, values, sparse);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slice(), &[34, 42]);
        assert_eq!(sparse_set.as_keys_slice(), &[5, 1]);
        assert_eq!(sparse_set.get(5), Some(&34));
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
        let mut sparse_set = SparseSet::new();
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
        let mut sparse_set = SparseSet::new();
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
        let mut sparse_set = SparseSet::new();
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
        sparse_set.insert(4, 34);
        sparse_set.insert(2, 42);
        sparse_set.insert(1, 69);
        sparse_set.insert(6, 228);
        sparse_set.insert(0, 666);

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, 69);

        let key = 6;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, 34);

        let key = 0;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, 666);

        let key = 3;
        let value = 0;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let key = 2;
        let value = 1;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, Some(42));
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
        sparse_set.insert(4, 34);
        sparse_set.insert(2, 42);
        sparse_set.insert(1, 69);
        sparse_set.insert(6, 228);
        sparse_set.insert(0, 666);

        let key = 1;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, 69);

        let key = 6;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, 34);

        let key = 0;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, 666);

        let key = 3;
        let value = 0;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let key = 2;
        let value = 1;
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, Some(42));
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
    fn five_items_remove_push() {
        let mut sparse_set = SparseSet::new();
        let _key0 = sparse_set.push(34);
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(69);
        let key3 = sparse_set.push(228);
        let key4 = sparse_set.push(666);

        let value = sparse_set.remove(key1).unwrap();
        assert_eq!(value, 42);

        let value = sparse_set.remove(key3).unwrap();
        assert_eq!(value, 228);

        let value = sparse_set.remove(key4).unwrap();
        assert_eq!(value, 666);

        let value = sparse_set.remove(key2).unwrap();
        assert_eq!(value, 69);

        let value = 0;
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let value = 1;
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let value = 10;
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn five_items_swap_remove_push() {
        let mut sparse_set = SparseSet::new();
        let _key0 = sparse_set.push(34);
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(69);
        let key3 = sparse_set.push(228);
        let key4 = sparse_set.push(666);

        let value = sparse_set.swap_remove(key1).unwrap();
        assert_eq!(value, 42);

        let value = sparse_set.swap_remove(key3).unwrap();
        assert_eq!(value, 228);

        let value = sparse_set.swap_remove(key4).unwrap();
        assert_eq!(value, 666);

        let value = sparse_set.swap_remove(key2).unwrap();
        assert_eq!(value, 69);

        let value = 0;
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let value = 1;
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some(&value));
        assert!(sparse_set.contains_key(key));

        let value = 10;
        let key = sparse_set.push(value);
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
    fn five_items_insert_truncate() {
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
    fn five_items_push_truncate() {
        let mut sparse_set = SparseSet::new();
        let key0 = sparse_set.push(34);
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(69);
        let key3 = sparse_set.push(228);
        let key4 = sparse_set.push(666);

        sparse_set.truncate(usize::MAX, 3);
        assert_eq!(sparse_set.sparse_len(), 3);
        assert_eq!(sparse_set.as_keys_slice(), &[key0, key1, key2]);
        assert_eq!(sparse_set.as_slice(), &[34, 42, 69]);

        assert_eq!(sparse_set.get(key0), Some(&34));
        assert_eq!(sparse_set.get(key1), Some(&42));
        assert_eq!(sparse_set.get(key2), Some(&69));
        assert_eq!(sparse_set.get(key3), None);
        assert_eq!(sparse_set.get(key4), None);

        sparse_set.truncate(1, usize::MAX);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_keys_slice(), &[key0]);
        assert_eq!(sparse_set.as_slice(), &[34]);

        assert_eq!(sparse_set.get(key0), Some(&34));
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
    fn five_items_entry() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, 42);
        sparse_set.insert(1, 228);
        sparse_set.insert(4, 69);
        sparse_set.insert(3, 666);
        sparse_set.insert(6, 34);

        let entry = sparse_set.entry(0);
        assert_eq!(entry.key(), 0);
        assert_eq!(entry.get(), None);

        let entry = entry.and_modify(|value| *value += 1);
        assert_eq!(entry.key(), 0);
        assert_eq!(entry.get(), None);

        let entry = entry.replace_key(1);
        assert_eq!(entry.key(), 1);
        assert_eq!(entry.get(), Some(&228));

        let value = entry.and_modify(|value| *value += 1).or_insert(47);
        assert_eq!(value, &229);
    }

    #[test]
    fn from_keys_values_iter() {
        let keys = [3, 10, 5, 10, 1, usize::MAX];
        let values = [34, 42, 69, 228, 666];

        let sparse_set: SparseSet<_> = keys.into_iter().zip(values).collect();
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

        let sparse_set: SparseSet<_> = keys.into_iter().zip(values).collect();
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
        let sparse_set: SparseSet<_> = values.into_iter().collect();

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
        let mut sparse_set = SparseSet::new();
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
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, 34);
        sparse_set.insert(1, 42);
        sparse_set.insert(4, 69);

        let values = [228, 666, 201];
        sparse_set.extend(values);

        assert_eq!(sparse_set.keys().as_slice(), &[2, 1, 4, 0, 3, 5]);
        assert_eq!(sparse_set.values().as_slice(), &[34, 42, 69, 228, 666, 201]);
    }
}
