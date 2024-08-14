use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};
use core::{
    cmp,
    fmt::{self, Debug, Display},
    mem::replace,
    ops::{Index, IndexMut},
};

use crate::{
    assert::{
        check_dense_index_bounds, check_equal_key, check_key_bounds, check_kv_same_capacity,
        check_kv_same_len, match_kv_same_kind, unwrap_dense_index_mut, unwrap_dense_value,
        unwrap_dense_value_mut, unwrap_next_vacant, unwrap_next_vacant_mut, unwrap_sparse_item_mut,
    },
    iter::{Drain, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut},
    key::{Epoch, Key},
    view::{EpochSparseView, EpochSparseViewMut},
    SparseItem, SparseItemKind,
};

fn extend_sparse<E>(sparse: &mut Vec<SparseItem<E>>, new_len: usize, sparse_vacant_head: &mut usize)
where
    E: Epoch,
{
    let old_len = sparse.len();
    if old_len >= new_len {
        return;
    }

    if *sparse_vacant_head < old_len {
        let mut last_vacant = *sparse_vacant_head;
        loop {
            let next_vacant = unwrap_next_vacant_mut(sparse[last_vacant].kind_mut());
            if *next_vacant == old_len {
                *next_vacant = new_len;
                break;
            }
            last_vacant = *next_vacant;
        }
    }

    let mut next_vacant = if *sparse_vacant_head < old_len {
        *sparse_vacant_head
    } else {
        new_len
    };
    let mut current_vacant = old_len;
    sparse.resize_with(new_len, || {
        let epoch = Default::default();
        let item = SparseItem::vacant(next_vacant, epoch);
        next_vacant = current_vacant;
        current_vacant += 1;
        item
    });

    *sparse_vacant_head = unwrap_next_vacant(sparse.last().unwrap().kind());
}

fn remove_from_vacant_list<E>(
    sparse: &mut [SparseItem<E>],
    sparse_vacant_head: &mut usize,
    sparse_index: usize,
    next_vacant: usize,
) {
    let vacant_to_fix = {
        let mut result = None;
        let mut next_vacant = *sparse_vacant_head;
        while next_vacant != sparse_index {
            result = Some(next_vacant);

            let vacant_item = sparse.index(next_vacant);
            next_vacant = unwrap_next_vacant(vacant_item.kind());
        }
        result
    };

    let vacant_to_fix = match vacant_to_fix {
        Some(vacant_to_fix) => {
            let vacant_item = sparse.index_mut(vacant_to_fix);
            unwrap_next_vacant_mut(vacant_item.kind_mut())
        }
        None => sparse_vacant_head,
    };
    *vacant_to_fix = next_vacant;
}

pub type SparseArena<T> = EpochSparseArena<usize, T>;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EpochSparseArena<K, V>
where
    K: Key,
{
    dense_keys: Vec<K>,
    dense_values: Vec<V>,
    sparse: Vec<SparseItem<K::Epoch>>,
    sparse_vacant_head: usize,
}

impl<K, V> EpochSparseArena<K, V>
where
    K: Key,
{
    #[inline]
    pub const fn new() -> Self {
        Self {
            dense_keys: Vec::new(),
            dense_values: Vec::new(),
            sparse: Vec::new(),
            sparse_vacant_head: 0,
        }
    }

    #[inline]
    pub fn with_capacity(dense: usize, sparse: usize) -> Self {
        Self {
            dense_keys: Vec::with_capacity(dense),
            dense_values: Vec::with_capacity(dense),
            sparse: Vec::with_capacity(sparse),
            sparse_vacant_head: 0,
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
            ..
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
            ..
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
            ..
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
            ..
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
            ..
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
            ..
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
    pub fn as_view(&self) -> EpochSparseView<'_, K, V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            ..
        } = self;

        EpochSparseView::new(dense_keys, dense_values, sparse)
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> EpochSparseViewMut<'_, K, V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            ..
        } = self;

        EpochSparseViewMut::new(dense_keys, dense_values, sparse)
    }

    #[inline]
    pub fn into_parts(self) -> (Vec<K>, Vec<V>, Vec<SparseItem<K::Epoch>>) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            ..
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
        let mut sparse_vacant_head = 0;
        for (dense_index, key) in keys.iter().enumerate() {
            let sparse_index = key.sparse_index();
            let epoch = key.epoch();
            let item = SparseItem::occupied(dense_index, epoch);

            if sparse_index >= sparse.len() {
                let new_len = sparse_index.saturating_add(1);
                extend_sparse(&mut sparse, new_len, &mut sparse_vacant_head);
            } else {
                let next_vacant = sparse.get(sparse_index).unwrap().next_vacant().unwrap();
                remove_from_vacant_list(
                    &mut sparse,
                    &mut sparse_vacant_head,
                    sparse_index,
                    next_vacant,
                );
            }
            sparse[sparse_index] = item;
        }

        Self {
            dense_keys: keys,
            dense_values: values,
            sparse,
            sparse_vacant_head,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            sparse_vacant_head,
        } = self;

        let sparse_index = key.sparse_index();
        match sparse.get_mut(sparse_index) {
            Some(sparse_item) if key.epoch() >= sparse_item.epoch => match sparse_item.kind {
                SparseItemKind::Occupied { dense_index } => {
                    let value_mut = unwrap_dense_value_mut(dense_values, dense_index);
                    let value = replace(value_mut, value);
                    sparse_item.epoch = key.epoch();
                    dense_keys[dense_index] = key;
                    Some(value)
                }
                SparseItemKind::Vacant { next_vacant } => {
                    remove_from_vacant_list(sparse, sparse_vacant_head, sparse_index, next_vacant);

                    check_kv_same_len(dense_keys.len(), dense_values.len());
                    dense_keys.push(key);
                    dense_values.push(value);
                    sparse[sparse_index] = SparseItem::occupied(dense_keys.len() - 1, key.epoch());

                    None
                }
            },
            Some(_) => None,
            None => {
                let new_sparse_len = sparse_index.saturating_add(1);
                extend_sparse(sparse, new_sparse_len, sparse_vacant_head);

                check_kv_same_len(dense_keys.len(), dense_values.len());
                dense_keys.push(key);
                dense_values.push(value);
                sparse[sparse_index] = SparseItem::occupied(dense_keys.len() - 1, key.epoch());

                None
            }
        }
    }

    pub fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            sparse_vacant_head,
        } = self;

        let sparse_index = key.sparse_index();
        match sparse.get_mut(sparse_index) {
            Some(sparse_item) if key.epoch() >= sparse_item.epoch => match sparse_item.kind {
                SparseItemKind::Occupied { dense_index } => {
                    let value_mut = unwrap_dense_value_mut(dense_values, dense_index);
                    let value = replace(value_mut, value);
                    sparse_item.epoch = key.epoch();
                    dense_keys[dense_index] = key;
                    Ok(Some(value))
                }
                SparseItemKind::Vacant { next_vacant } => {
                    remove_from_vacant_list(sparse, sparse_vacant_head, sparse_index, next_vacant);

                    check_kv_same_len(dense_keys.len(), dense_values.len());
                    dense_keys.try_reserve(1)?;
                    dense_values.try_reserve(1)?;

                    dense_keys.push(key);
                    dense_values.push(value);
                    sparse[sparse_index] = SparseItem::occupied(dense_keys.len() - 1, key.epoch());

                    Ok(None)
                }
            },
            Some(_) => Ok(None),
            None => {
                let new_sparse_len = sparse_index.saturating_add(1);
                sparse.try_reserve(new_sparse_len.saturating_sub(sparse.len()))?;
                extend_sparse(sparse, new_sparse_len, sparse_vacant_head);

                check_kv_same_len(dense_keys.len(), dense_values.len());
                dense_keys.try_reserve(1)?;
                dense_values.try_reserve(1)?;

                dense_keys.push(key);
                dense_values.push(value);
                sparse[sparse_index] = SparseItem::occupied(dense_keys.len() - 1, key.epoch());

                Ok(None)
            }
        }
    }

    pub fn push(&mut self, value: V) -> K {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            sparse_vacant_head,
        } = self;

        if let Some(sparse_item) = sparse.get_mut(*sparse_vacant_head) {
            let next_vacant = unwrap_next_vacant(sparse_item.kind());

            let key = K::new(*sparse_vacant_head, sparse_item.epoch);
            let sparse_item_kind = SparseItemKind::occupied(dense_keys.len());

            check_kv_same_len(dense_keys.len(), dense_values.len());
            dense_keys.push(key);
            dense_values.push(value);

            sparse_item.kind = sparse_item_kind;
            *sparse_vacant_head = next_vacant;

            return key;
        }

        let key = Key::new(*sparse_vacant_head, Default::default());
        let sparse_item = SparseItem::occupied(dense_keys.len(), Default::default());

        check_kv_same_len(dense_keys.len(), dense_values.len());
        dense_keys.push(key);
        dense_values.push(value);

        sparse.push(sparse_item);
        *sparse_vacant_head = dense_keys.len();

        key
    }

    pub fn try_push(&mut self, value: V) -> Result<K, TryReserveError> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            sparse_vacant_head,
        } = self;

        if let Some(sparse_item) = sparse.get_mut(*sparse_vacant_head) {
            let next_vacant = unwrap_next_vacant(sparse_item.kind());

            let key = K::new(*sparse_vacant_head, sparse_item.epoch);
            let sparse_item_kind = SparseItemKind::occupied(dense_keys.len());

            check_kv_same_len(dense_keys.len(), dense_values.len());
            dense_keys.try_reserve(1)?;
            dense_values.try_reserve(1)?;

            dense_keys.push(key);
            dense_values.push(value);

            sparse_item.kind = sparse_item_kind;
            *sparse_vacant_head = next_vacant;

            return Ok(key);
        }

        let key = Key::new(*sparse_vacant_head, Default::default());
        let sparse_item = SparseItem::occupied(dense_keys.len(), Default::default());

        check_kv_same_len(dense_keys.len(), dense_values.len());
        dense_keys.try_reserve(1)?;
        dense_values.try_reserve(1)?;
        sparse.try_reserve(1)?;

        dense_keys.push(key);
        dense_values.push(value);

        sparse.push(sparse_item);
        *sparse_vacant_head = dense_keys.len();

        Ok(key)
    }

    pub fn swap(&mut self, first_key: K, second_key: K) {
        let mut view_mut = self.as_mut_view();
        view_mut.swap(first_key, second_key)
    }

    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let mut view_mut = self.as_mut_view();
        view_mut.swap_keys(first_key, second_key)
    }

    pub fn swap_remove(&mut self, key: K) -> Option<V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            sparse_vacant_head,
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
            match sparse_item.kind_mut() {
                SparseItemKind::Occupied { dense_index: index } => *index = dense_index,
                SparseItemKind::Vacant { next_vacant } => *next_vacant = dense_index,
            }
        }
        sparse[sparse_index] = SparseItem::vacant(*sparse_vacant_head, key.epoch().next());
        *sparse_vacant_head = sparse_index;

        Some(value)
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            sparse_vacant_head,
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
        sparse[sparse_index] = SparseItem::vacant(*sparse_vacant_head, key.epoch().next());
        *sparse_vacant_head = sparse_index;

        Some(value)
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            sparse_vacant_head,
        } = self;

        let key = dense_keys.pop();
        let value = dense_values.pop();
        let (key, value) = match_kv_same_kind(key, value)?;

        let sparse_index = key.sparse_index();
        check_key_bounds(sparse_index, sparse.len());

        sparse[sparse_index] = SparseItem::vacant(*sparse_vacant_head, key.epoch().next());
        *sparse_vacant_head = sparse_index;

        Some((key, value))
    }

    pub fn invalidate_epoch(&mut self, key: K) -> Option<K> {
        let mut view_mut = self.as_mut_view();
        view_mut.invalidate_epoch(key)
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
            sparse_vacant_head,
        } = self;

        let keys = dense_keys.drain(..);
        let values = dense_values.drain(..);
        sparse.clear();
        *sparse_vacant_head = 0;

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
        let mut view_mut = self.as_mut_view();
        view_mut.sort()
    }

    #[inline]
    pub fn sort_keys(&mut self) {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_keys()
    }

    #[inline]
    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut((K, &V), (K, &V)) -> cmp::Ordering,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_by(f)
    }

    #[inline]
    pub fn sort_by_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, &V)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_by_key(f)
    }

    #[inline]
    pub fn sort_by_cached_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, &V)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_by_cached_key(f)
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        V: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_unstable()
    }

    #[inline]
    pub fn sort_keys_unstable(&mut self) {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_keys_unstable()
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, f: F)
    where
        F: FnMut((K, &V), (K, &V)) -> cmp::Ordering,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_unstable_by(f)
    }

    #[inline]
    pub fn sort_unstable_by_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, &V)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_unstable_by_key(f)
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<&V> {
        let view = self.as_view();
        view.get(key)
    }

    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let view_mut = self.as_mut_view();
        view_mut.into_get_mut(key)
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: usize) -> Option<(K, &V)> {
        let view = self.as_view();
        view.get_with_key(sparse_index)
    }

    #[inline]
    pub fn get_mut_with_key(&mut self, sparse_index: usize) -> Option<(K, &mut V)> {
        let view_mut = self.as_mut_view();
        view_mut.into_get_mut_with_key(sparse_index)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<K::Epoch> {
        let view = self.as_view();
        view.get_epoch(sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let view = self.as_view();
        view.contains_key(key)
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

    pub fn clear(&mut self) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
            sparse_vacant_head,
        } = self;

        dense_keys.clear();
        dense_values.clear();
        sparse.clear();
        *sparse_vacant_head = 0;
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        let view = self.as_view();
        view.keys()
    }

    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        let Self { dense_keys, .. } = self;

        let keys = dense_keys.into_iter();
        IntoKeys::new(keys)
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        let view = self.as_view();
        view.values()
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        let view_mut = self.as_mut_view();
        view_mut.into_values_mut()
    }

    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        let Self { dense_values, .. } = self;

        let values = dense_values.into_iter();
        IntoValues::new(values)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        let view = self.as_view();
        view.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        let view_mut = self.as_mut_view();
        view_mut.into_iter()
    }
}

impl<K, V> Index<K> for EpochSparseArena<K, V>
where
    K: Key + Display,
{
    type Output = V;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        let view = self.as_view();
        view.into_index(key)
    }
}

impl<K, V> IndexMut<K> for EpochSparseArena<K, V>
where
    K: Key + Display,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        let view_mut = self.as_mut_view();
        view_mut.into_index_mut(key)
    }
}

impl<K, V> AsRef<[V]> for EpochSparseArena<K, V>
where
    K: Key,
{
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<K, V> AsMut<[V]> for EpochSparseArena<K, V>
where
    K: Key,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [V] {
        self.as_mut_slice()
    }
}

impl<K, V> AsRef<EpochSparseArena<K, V>> for EpochSparseArena<K, V>
where
    K: Key,
{
    #[inline]
    fn as_ref(&self) -> &EpochSparseArena<K, V> {
        self
    }
}

impl<K, V> AsMut<EpochSparseArena<K, V>> for EpochSparseArena<K, V>
where
    K: Key,
{
    #[inline]
    fn as_mut(&mut self) -> &mut EpochSparseArena<K, V> {
        self
    }
}

impl<'a, K, V> IntoIterator for &'a EpochSparseArena<K, V>
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

impl<'a, K, V> IntoIterator for &'a mut EpochSparseArena<K, V>
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

impl<K, V> IntoIterator for EpochSparseArena<K, V>
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

impl<K, V> FromIterator<(K, V)> for EpochSparseArena<K, V>
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

impl<K, V> FromIterator<V> for EpochSparseArena<K, V>
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
        let sparse_vacant_head = len;

        Self {
            dense_keys,
            dense_values,
            sparse,
            sparse_vacant_head,
        }
    }
}

impl<K, V> Extend<(K, V)> for EpochSparseArena<K, V>
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

impl<K, V> Extend<V> for EpochSparseArena<K, V>
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

        for value in iter {
            self.push(value);
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
    sparse_set: &'a mut EpochSparseArena<K, V>,
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
    sparse_set: &'a mut EpochSparseArena<K, V>,
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

#[cfg(test)]
mod tests {
    use core::{mem::forget, ops::Not};

    use crate::prelude::*;

    type Key = EpochKey;

    #[test]
    fn empty() {
        let sparse_arena = SparseArena::<i32>::new();
        assert!(sparse_arena.is_empty());
    }

    #[test]
    fn with_capacity() {
        let sparse_arena = SparseArena::<i32>::with_capacity(10, 10);
        assert!(sparse_arena.is_empty());
        assert!(sparse_arena.capacity() >= 10);
        assert!(sparse_arena.sparse_capacity() >= 10);
    }

    #[test]
    fn empty_parts() {
        let sparse_arena = SparseArena::<i32>::new();

        let (keys, values, sparse) = sparse_arena.into_parts();
        assert_eq!(keys.len(), 0);
        assert_eq!(values.len(), 0);
        assert_eq!(sparse.len(), 0);

        let sparse_arena = SparseArena::from_parts(keys, values, sparse);
        assert_eq!(sparse_arena.len(), 0);
    }

    #[test]
    fn empty_keys() {
        let sparse_arena = SparseArena::<i32>::new();

        let keys = sparse_arena.keys();
        assert_eq!(keys.len(), 0);
        assert_eq!(keys.as_slice(), &[]);
    }

    #[test]
    fn empty_into_keys() {
        let sparse_arena = SparseArena::<i32>::new();

        let keys = sparse_arena.into_keys();
        assert_eq!(keys.len(), 0);
        assert_eq!(keys.as_slice(), &[]);
    }

    #[test]
    fn empty_values() {
        let sparse_arena = SparseArena::<i32>::new();

        let values = sparse_arena.values();
        assert_eq!(values.len(), 0);
        assert_eq!(values.as_slice(), &[]);
    }

    #[test]
    fn empty_values_mut() {
        let mut sparse_arena = SparseArena::<i32>::new();
        let values_mut = sparse_arena.values_mut();

        assert_eq!(values_mut.len(), 0);
        assert_eq!(values_mut.into_slice(), &mut []);
    }

    #[test]
    fn empty_into_values() {
        let sparse_arena = SparseArena::<i32>::new();

        let values = sparse_arena.into_values();
        assert_eq!(values.len(), 0);
        assert_eq!(values.as_slice(), &[]);
    }

    #[test]
    fn empty_iter() {
        let sparse_arena = SparseArena::<i32>::new();

        let iter = sparse_arena.iter();
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.as_keys_slice(), &[]);
        assert_eq!(iter.as_values_slice(), &[]);
    }

    #[test]
    fn empty_iter_mut() {
        let mut sparse_arena = SparseArena::<i32>::new();
        let iter_mut = sparse_arena.iter_mut();

        assert_eq!(iter_mut.len(), 0);
        assert_eq!(iter_mut.as_keys_slice(), &[]);
        assert_eq!(iter_mut.into_values_slice(), &mut []);
    }

    #[test]
    fn empty_into_iter() {
        let sparse_arena = SparseArena::<i32>::new();
        let into_iter = sparse_arena.into_iter();

        assert_eq!(into_iter.len(), 0);
        assert_eq!(into_iter.as_keys_slice(), &[]);
        assert_eq!(into_iter.as_values_slice(), &[]);
    }

    #[test]
    fn empty_insert_one() {
        let mut sparse_arena = SparseArena::new();
        let previous = sparse_arena.insert(0, 42);
        assert_eq!(previous, None);

        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert!(sparse_arena.contains_key(0));
    }

    #[test]
    fn with_capacity_insert_one() {
        let mut sparse_arena = SparseArena::with_capacity(10, 10);
        let previous = sparse_arena.insert(0, 42);
        assert_eq!(previous, None);

        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert!(sparse_arena.contains_key(0));
    }

    #[test]
    fn empty_insert_one_mutate() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);
        *sparse_arena.get_mut(0).unwrap() = 43;

        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(0), Some(&43));
        assert!(sparse_arena.contains_key(0));
    }

    #[test]
    fn with_capacity_insert_one_mutate() {
        let mut sparse_arena = SparseArena::with_capacity(10, 10);
        sparse_arena.insert(0, 42);
        *sparse_arena.get_mut(0).unwrap() = 43;

        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(0), Some(&43));
        assert!(sparse_arena.contains_key(0));
    }

    #[test]
    fn empty_insert_far() {
        let mut sparse_arena = SparseArena::new();

        let (key, value) = (3, 42);
        sparse_arena.insert(key, value);

        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));

        let (key, value) = (6, 69);
        sparse_arena.insert(key, value);

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));
    }

    #[test]
    fn empty_insert_far_remove() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(3, 42);
        sparse_arena.insert(1, 69);

        let key = 3;
        let value = sparse_arena.remove(key).unwrap();

        assert_eq!(value, 42);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());

        let key = 1;
        let value = sparse_arena.remove(key).unwrap();

        assert_eq!(value, 69);
        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());
    }

    #[test]
    fn empty_push() {
        let mut sparse_arena = SparseArena::new();

        let key = sparse_arena.push(42);
        assert_eq!(key, 0);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(key), Some(&42));
        assert!(sparse_arena.contains_key(key));
    }

    #[test]
    fn empty_pop() {
        let mut sparse_arena = SparseArena::<i32>::new();

        let popped = sparse_arena.pop();
        assert_eq!(popped, None);
        assert_eq!(sparse_arena.len(), 0);
    }

    #[test]
    fn one_item_insert_remove_one() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let removed = sparse_arena.remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.get(0), None);
        assert!(sparse_arena.contains_key(0).not());
    }

    #[test]
    fn one_item_insert_remove_one_epoch() {
        let mut sparse_arena = EpochSparseArena::new();

        let key = Key::new(0, 1);
        sparse_arena.insert(key, 42);

        let removed = sparse_arena.remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());

        assert_eq!(
            sparse_arena.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());
    }

    #[test]
    fn one_item_insert_swap_remove_one() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let removed = sparse_arena.swap_remove(0);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.get(0), None);
        assert!(sparse_arena.contains_key(0).not());
    }

    #[test]
    fn one_item_insert_swap_remove_one_epoch() {
        let mut sparse_arena = EpochSparseArena::new();

        let key = Key::new(0, 1);
        sparse_arena.insert(key, 42);

        let removed = sparse_arena.swap_remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());

        assert_eq!(
            sparse_arena.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());
    }

    #[test]
    fn one_item_push_remove_one() {
        let mut sparse_arena = SparseArena::new();
        let key = sparse_arena.push(42);

        let removed = sparse_arena.remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());
    }

    #[test]
    fn one_item_push_remove_one_epoch() {
        let mut sparse_arena = EpochSparseArena::<Key, _>::new();
        let key = sparse_arena.push(42);

        let removed = sparse_arena.remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());

        assert_eq!(
            sparse_arena.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());
    }

    #[test]
    fn one_item_push_swap_remove_one() {
        let mut sparse_arena = SparseArena::new();
        let key = sparse_arena.push(42);

        let removed = sparse_arena.swap_remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());
    }

    #[test]
    fn one_item_push_swap_remove_one_epoch() {
        let mut sparse_arena = EpochSparseArena::<Key, _>::new();
        let key = sparse_arena.push(42);

        let removed = sparse_arena.swap_remove(key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());

        assert_eq!(
            sparse_arena.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_arena.get(key), None);
        assert!(sparse_arena.contains_key(key).not());
    }

    #[test]
    fn one_item_swap() {
        let mut sparse_arena = SparseArena::new();
        let key = sparse_arena.push(42);
        assert_eq!(key, 0);

        sparse_arena.swap(0, 0);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.as_slice(), &[42]);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert!(sparse_arena.contains_key(0));

        sparse_arena.swap(0, 1);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.as_slice(), &[42]);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert!(sparse_arena.contains_key(0));
    }

    #[test]
    fn one_item_swap_keys() {
        let mut sparse_arena = SparseArena::new();
        let key = sparse_arena.push(42);
        assert_eq!(key, 0);

        sparse_arena.swap_keys(0, 0);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.as_slice(), &[42]);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert!(sparse_arena.contains_key(0));

        sparse_arena.swap_keys(0, 1);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.as_slice(), &[42]);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert!(sparse_arena.contains_key(0));
    }

    #[test]
    fn one_item_parts() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 42);

        let (keys, values, sparse) = sparse_arena.into_parts();
        assert_eq!(keys, &[2]);
        assert_eq!(values, &[42]);
        assert_eq!(
            sparse,
            &[
                SparseItem::vacant(3, ()),
                SparseItem::vacant(0, ()),
                SparseItem::occupied(0, ()),
            ]
        );

        let sparse_arena = SparseArena::from_parts(keys, values, sparse);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.as_slice(), &[42]);
        assert_eq!(sparse_arena.as_keys_slice(), &[2]);
        assert_eq!(sparse_arena.get(2), Some(&42));
    }

    #[test]
    fn one_item_keys() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let keys = sparse_arena.keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.as_slice(), &[0]);
    }

    #[test]
    fn one_item_into_keys() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let keys = sparse_arena.into_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.as_slice(), &[0]);
    }

    #[test]
    fn one_item_values() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let values = sparse_arena.values();
        assert_eq!(values.len(), 1);
        assert_eq!(values.as_slice(), &[42]);
    }

    #[test]
    fn one_item_values_mut() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let values_mut = sparse_arena.values_mut();
        assert_eq!(values_mut.len(), 1);
        assert_eq!(values_mut.into_slice(), &mut [42]);
    }

    #[test]
    fn one_item_into_values() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let values = sparse_arena.into_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values.as_slice(), &[42]);
    }

    #[test]
    fn one_item_iter() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let iter = sparse_arena.iter();
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.as_keys_slice(), &[0]);
        assert_eq!(iter.as_values_slice(), &[42]);
    }

    #[test]
    fn one_item_iter_mut() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let iter_mut = sparse_arena.iter_mut();
        assert_eq!(iter_mut.len(), 1);
        assert_eq!(iter_mut.as_keys_slice(), &[0]);
        assert_eq!(iter_mut.into_values_slice(), &mut [42]);
    }

    #[test]
    fn one_item_into_iter() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);

        let into_iter = sparse_arena.into_iter();
        assert_eq!(into_iter.len(), 1);
        assert_eq!(into_iter.as_keys_slice(), &[0]);
        assert_eq!(into_iter.as_values_slice(), &[42]);
    }

    #[test]
    fn two_items_insert_first() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);
        sparse_arena.insert(1, 69);

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert_eq!(sparse_arena.get(1), Some(&69));

        let previous = sparse_arena.insert(0, 34);
        assert_eq!(previous, Some(42));

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(0), Some(&34));
        assert_eq!(sparse_arena.get(1), Some(&69));
        assert!(sparse_arena.contains_key(0));
        assert!(sparse_arena.contains_key(1));
    }

    #[test]
    fn two_items_insert_first_epoch() {
        let mut sparse_arena = EpochSparseArena::new();

        let first_key = Key::new(0, 3);
        sparse_arena.insert(first_key, 42);

        let second_key = Key::new(1, 0);
        sparse_arena.insert(second_key, 69);

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), Some(&69));

        let first_key = Key::new(first_key.sparse_index(), first_key.epoch().next());
        let previous = sparse_arena.insert(first_key, 34);
        assert_eq!(previous, Some(42));

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(first_key), Some(&34));
        assert_eq!(sparse_arena.get(second_key), Some(&69));
        assert!(sparse_arena.contains_key(first_key));
        assert!(sparse_arena.contains_key(second_key));
    }

    #[test]
    fn two_items_insert_second() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);
        sparse_arena.insert(1, 69);

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert_eq!(sparse_arena.get(1), Some(&69));

        let previous = sparse_arena.insert(1, 34);
        assert_eq!(previous, Some(69));

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert_eq!(sparse_arena.get(1), Some(&34));
        assert!(sparse_arena.contains_key(0));
        assert!(sparse_arena.contains_key(1));
    }

    #[test]
    fn two_items_remove_first() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), Some(&69));

        let removed = sparse_arena.remove(first_key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(first_key), None);
        assert_eq!(sparse_arena.get(second_key), Some(&69));
        assert!(sparse_arena.contains_key(first_key).not());
        assert!(sparse_arena.contains_key(second_key));
    }

    #[test]
    fn two_items_swap_remove_first() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), Some(&69));

        let removed = sparse_arena.swap_remove(first_key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(first_key), None);
        assert_eq!(sparse_arena.get(second_key), Some(&69));
        assert!(sparse_arena.contains_key(first_key).not());
        assert!(sparse_arena.contains_key(second_key));
    }

    #[test]
    fn two_items_remove_second() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), Some(&69));

        let removed = sparse_arena.remove(second_key);
        assert_eq!(removed, Some(69));

        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), None);
        assert!(sparse_arena.contains_key(first_key));
        assert!(sparse_arena.contains_key(second_key).not());
    }

    #[test]
    fn two_items_swap_remove_second() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), Some(&69));

        let removed = sparse_arena.swap_remove(second_key);
        assert_eq!(removed, Some(69));

        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), None);
        assert!(sparse_arena.contains_key(first_key));
        assert!(sparse_arena.contains_key(second_key).not());
    }

    #[test]
    fn two_items_remove_one_insert_one() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);
        sparse_arena.insert(1, 69);

        let removed = sparse_arena.remove(0);
        assert_eq!(removed, Some(42));
        assert_eq!(sparse_arena.get(0), None);

        sparse_arena.insert(0, 34);
        assert_eq!(sparse_arena.get(0), Some(&34));
        assert_eq!(sparse_arena.get(1), Some(&69));
        assert!(sparse_arena.contains_key(0));
        assert!(sparse_arena.contains_key(1));
    }

    #[test]
    fn two_items_swap_remove_one_insert_one() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(0, 42);
        sparse_arena.insert(1, 69);

        let removed = sparse_arena.swap_remove(0);
        assert_eq!(removed, Some(42));
        assert_eq!(sparse_arena.get(0), None);

        sparse_arena.insert(0, 34);
        assert_eq!(sparse_arena.get(0), Some(&34));
        assert_eq!(sparse_arena.get(1), Some(&69));
        assert!(sparse_arena.contains_key(0));
        assert!(sparse_arena.contains_key(1));
    }

    #[test]
    fn two_items_remove_one_push_one() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        let removed = sparse_arena.remove(first_key);
        assert_eq!(removed, Some(42));
        assert_eq!(sparse_arena.get(first_key), None);

        let key = sparse_arena.push(34);
        assert_eq!(key, first_key);

        assert_eq!(sparse_arena.get(first_key), Some(&34));
        assert_eq!(sparse_arena.get(second_key), Some(&69));
        assert!(sparse_arena.contains_key(first_key));
        assert!(sparse_arena.contains_key(second_key));
    }

    #[test]
    fn two_items_swap_remove_one_push_one() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        let removed = sparse_arena.swap_remove(first_key);
        assert_eq!(removed, Some(42));
        assert_eq!(sparse_arena.get(first_key), None);

        let key = sparse_arena.push(34);
        assert_eq!(key, first_key);

        assert_eq!(sparse_arena.get(first_key), Some(&34));
        assert_eq!(sparse_arena.get(second_key), Some(&69));
        assert!(sparse_arena.contains_key(first_key));
        assert!(sparse_arena.contains_key(second_key));
    }

    #[test]
    fn two_items_swap() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        sparse_arena.swap(first_key, first_key);
        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.as_slice(), &[42, 69]);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert_eq!(sparse_arena.get(1), Some(&69));

        sparse_arena.swap(first_key, second_key);
        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.as_slice(), &[69, 42]);
        assert_eq!(sparse_arena.get(0), Some(&69));
        assert_eq!(sparse_arena.get(1), Some(&42));

        sparse_arena.swap(second_key, second_key);
        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.as_slice(), &[69, 42]);
        assert_eq!(sparse_arena.get(0), Some(&69));
        assert_eq!(sparse_arena.get(1), Some(&42));
    }

    #[test]
    fn two_items_swap_keys() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        sparse_arena.swap_keys(first_key, first_key);
        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.as_slice(), &[42, 69]);
        assert_eq!(sparse_arena.get(0), Some(&42));
        assert_eq!(sparse_arena.get(1), Some(&69));

        sparse_arena.swap_keys(first_key, second_key);
        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.as_slice(), &[42, 69]);
        assert_eq!(sparse_arena.get(0), Some(&69));
        assert_eq!(sparse_arena.get(1), Some(&42));

        sparse_arena.swap_keys(second_key, second_key);
        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.as_slice(), &[42, 69]);
        assert_eq!(sparse_arena.get(0), Some(&69));
        assert_eq!(sparse_arena.get(1), Some(&42));
    }

    #[test]
    fn two_items_insert_pop() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(5, 42);
        sparse_arena.insert(2, 69);

        let popped = sparse_arena.pop();
        assert_eq!(popped, Some((2, 69)));
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(5), Some(&42));
        assert_eq!(sparse_arena.get(2), None);
    }

    #[test]
    fn two_items_push_pop() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        let popped = sparse_arena.pop();
        assert_eq!(popped, Some((second_key, 69)));
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), None);
    }

    #[test]
    fn two_items_insert_pop_epoch() {
        let mut sparse_arena = EpochSparseArena::new();

        let first_key = Key::new(5, 1);
        sparse_arena.insert(first_key, 42);

        let second_key = Key::new(2, 0);
        sparse_arena.insert(second_key, 69);

        let popped = sparse_arena.pop();
        assert_eq!(popped, Some((second_key, 69)));
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), None);

        assert_eq!(
            sparse_arena.get_epoch(second_key.sparse_index()),
            Some(second_key.epoch().next()),
        );
    }

    #[test]
    fn two_items_push_pop_epoch() {
        let mut sparse_arena = EpochSparseArena::<Key, _>::new();
        let first_key = sparse_arena.push(42);
        let second_key = sparse_arena.push(69);

        let popped = sparse_arena.pop();
        assert_eq!(popped, Some((second_key, 69)));
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.get(first_key), Some(&42));
        assert_eq!(sparse_arena.get(second_key), None);

        assert_eq!(
            sparse_arena.get_epoch(second_key.sparse_index()),
            Some(second_key.epoch().next()),
        );
    }

    #[test]
    fn two_items_invalidate_epoch() {
        let mut sparse_arena = EpochSparseArena::new();

        let first_key = Key::new(5, 1);
        sparse_arena.insert(first_key, 42);

        let second_key = Key::new(2, 0);
        sparse_arena.insert(second_key, 69);

        let new_first_key = sparse_arena
            .invalidate_epoch(first_key)
            .expect("first key should be present");
        assert_eq!(new_first_key.sparse_index(), first_key.sparse_index());
        assert_eq!(new_first_key.epoch(), &first_key.epoch().next());
        assert_eq!(new_first_key, Key::new(5, 2));
        assert_eq!(sparse_arena.get(first_key), None);
        assert_eq!(sparse_arena.get(new_first_key), Some(&42));

        let new_second_key = sparse_arena
            .invalidate_epoch(second_key)
            .expect("second key should be present");
        assert_eq!(new_second_key.sparse_index(), second_key.sparse_index());
        assert_eq!(new_second_key.epoch(), &second_key.epoch().next());
        assert_eq!(new_second_key, Key::new(2, 1));
        assert_eq!(sparse_arena.get(second_key), None);
        assert_eq!(sparse_arena.get(new_second_key), Some(&69));
    }

    #[test]
    fn three_items_insert_remove_middle() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);
        println!("{:#?}", sparse_arena);

        let removed = sparse_arena.remove(2);
        assert_eq!(removed, Some(34));

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(2), None);
        assert_eq!(sparse_arena.get(1), Some(&42));
        assert_eq!(sparse_arena.get(5), Some(&69));
        assert!(sparse_arena.contains_key(2).not());
        assert!(sparse_arena.contains_key(1));
        assert!(sparse_arena.contains_key(5));
    }

    #[test]
    fn three_items_push_remove_middle() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(34);
        let middle_key = sparse_arena.push(42);
        let last_key = sparse_arena.push(69);

        let removed = sparse_arena.remove(middle_key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(first_key), Some(&34));
        assert_eq!(sparse_arena.get(middle_key), None);
        assert_eq!(sparse_arena.get(last_key), Some(&69));
        assert!(sparse_arena.contains_key(first_key));
        assert!(sparse_arena.contains_key(middle_key).not());
        assert!(sparse_arena.contains_key(last_key));
    }

    #[test]
    fn three_items_swap_remove_middle() {
        let mut sparse_arena = SparseArena::new();
        let first_key = sparse_arena.push(34);
        let middle_key = sparse_arena.push(42);
        let last_key = sparse_arena.push(69);

        let removed = sparse_arena.swap_remove(middle_key);
        assert_eq!(removed, Some(42));

        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.get(first_key), Some(&34));
        assert_eq!(sparse_arena.get(middle_key), None);
        assert_eq!(sparse_arena.get(last_key), Some(&69));
        assert!(sparse_arena.contains_key(first_key));
        assert!(sparse_arena.contains_key(middle_key).not());
        assert!(sparse_arena.contains_key(last_key));
    }

    #[test]
    fn three_items_parts() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let (mut keys, values, sparse) = sparse_arena.into_parts();
        assert_eq!(keys, &[2, 1, 5]);
        assert_eq!(values, &[34, 42, 69]);
        assert_eq!(
            sparse,
            &[
                SparseItem::vacant(6, ()),
                SparseItem::occupied(1, ()),
                SparseItem::occupied(0, ()),
                SparseItem::vacant(0, ()),
                SparseItem::vacant(3, ()),
                SparseItem::occupied(2, ()),
            ]
        );

        keys.swap_remove(0);
        let sparse_arena = SparseArena::from_parts(keys, values, sparse);
        assert_eq!(sparse_arena.len(), 2);
        assert_eq!(sparse_arena.as_slice(), &[34, 42]);
        assert_eq!(sparse_arena.as_keys_slice(), &[5, 1]);
        assert_eq!(sparse_arena.get(5), Some(&34));
    }

    #[test]
    fn three_items_keys() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let keys = sparse_arena.keys();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys.as_slice(), &[2, 1, 5]);
    }

    #[test]
    fn three_items_into_keys() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let keys = sparse_arena.into_keys();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys.as_slice(), &[2, 1, 5]);
    }

    #[test]
    fn three_items_values() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let values = sparse_arena.values();
        assert_eq!(values.len(), 3);
        assert_eq!(values.as_slice(), &[34, 42, 69]);
    }

    #[test]
    fn three_items_values_mut() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let values_mut = sparse_arena.values_mut();
        assert_eq!(values_mut.len(), 3);
        assert_eq!(values_mut.into_slice(), &mut [34, 42, 69]);
    }

    #[test]
    fn three_items_into_values() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let values = sparse_arena.into_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values.as_slice(), &[34, 42, 69]);
    }

    #[test]
    fn three_items_iter() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let iter = sparse_arena.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.as_keys_slice(), &[2, 1, 5]);
        assert_eq!(iter.as_values_slice(), &[34, 42, 69]);
    }

    #[test]
    fn three_items_iter_mut() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let iter_mut = sparse_arena.iter_mut();
        assert_eq!(iter_mut.len(), 3);
        assert_eq!(iter_mut.as_keys_slice(), &[2, 1, 5]);
        assert_eq!(iter_mut.into_values_slice(), &mut [34, 42, 69]);
    }

    #[test]
    fn three_items_into_iter() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let into_iter = sparse_arena.into_iter();
        assert_eq!(into_iter.len(), 3);
        assert_eq!(into_iter.as_keys_slice(), &[2, 1, 5]);
        assert_eq!(into_iter.as_values_slice(), &[34, 42, 69]);
    }

    #[test]
    fn five_items_remove_insert() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(4, 34);
        sparse_arena.insert(2, 42);
        sparse_arena.insert(1, 69);
        sparse_arena.insert(6, 228);
        sparse_arena.insert(0, 666);

        let key = 1;
        let value = sparse_arena.remove(key).unwrap();
        assert_eq!(value, 69);

        let key = 6;
        let value = sparse_arena.remove(key).unwrap();
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_arena.remove(key).unwrap();
        assert_eq!(value, 34);

        let key = 0;
        let value = sparse_arena.remove(key).unwrap();
        assert_eq!(value, 666);

        let key = 3;
        let value = 0;
        let previous = sparse_arena.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));

        let key = 2;
        let value = 1;
        let previous = sparse_arena.insert(key, value);

        assert_eq!(previous, Some(42));
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));

        let key = 4;
        let value = 10;
        let previous = sparse_arena.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));
    }

    #[test]
    fn five_items_swap_remove_insert() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(4, 34);
        sparse_arena.insert(2, 42);
        sparse_arena.insert(1, 69);
        sparse_arena.insert(6, 228);
        sparse_arena.insert(0, 666);

        let key = 1;
        let value = sparse_arena.swap_remove(key).unwrap();
        assert_eq!(value, 69);

        let key = 6;
        let value = sparse_arena.swap_remove(key).unwrap();
        assert_eq!(value, 228);

        let key = 4;
        let value = sparse_arena.swap_remove(key).unwrap();
        assert_eq!(value, 34);

        let key = 0;
        let value = sparse_arena.swap_remove(key).unwrap();
        assert_eq!(value, 666);

        let key = 3;
        let value = 0;
        let previous = sparse_arena.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));

        let key = 2;
        let value = 1;
        let previous = sparse_arena.insert(key, value);

        assert_eq!(previous, Some(42));
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));

        let key = 4;
        let value = 10;
        let previous = sparse_arena.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));
    }

    #[test]
    fn five_items_remove_push() {
        let mut sparse_arena = SparseArena::new();
        let _key0 = sparse_arena.push(34);
        let key1 = sparse_arena.push(42);
        let key2 = sparse_arena.push(69);
        let key3 = sparse_arena.push(228);
        let key4 = sparse_arena.push(666);

        let value = sparse_arena.remove(key1).unwrap();
        assert_eq!(value, 42);

        let value = sparse_arena.remove(key3).unwrap();
        assert_eq!(value, 228);

        let value = sparse_arena.remove(key4).unwrap();
        assert_eq!(value, 666);

        let value = sparse_arena.remove(key2).unwrap();
        assert_eq!(value, 69);

        let value = 0;
        let key = sparse_arena.push(value);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));

        let value = 1;
        let key = sparse_arena.push(value);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));

        let value = 10;
        let key = sparse_arena.push(value);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));
    }

    #[test]
    fn five_items_swap_remove_push() {
        let mut sparse_arena = SparseArena::new();
        let _key0 = sparse_arena.push(34);
        let key1 = sparse_arena.push(42);
        let key2 = sparse_arena.push(69);
        let key3 = sparse_arena.push(228);
        let key4 = sparse_arena.push(666);

        let value = sparse_arena.swap_remove(key1).unwrap();
        assert_eq!(value, 42);

        let value = sparse_arena.swap_remove(key3).unwrap();
        assert_eq!(value, 228);

        let value = sparse_arena.swap_remove(key4).unwrap();
        assert_eq!(value, 666);

        let value = sparse_arena.swap_remove(key2).unwrap();
        assert_eq!(value, 69);

        let value = 0;
        let key = sparse_arena.push(value);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));

        let value = 1;
        let key = sparse_arena.push(value);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));

        let value = 10;
        let key = sparse_arena.push(value);
        assert_eq!(sparse_arena.get(key), Some(&value));
        assert!(sparse_arena.contains_key(key));
    }

    #[test]
    fn five_items_retain() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(8, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(4, 69);
        sparse_arena.insert(3, 228);
        sparse_arena.insert(6, 666);

        sparse_arena.retain(|key, _| key % 2 == 0);
        assert_eq!(sparse_arena.len(), 3);
        assert_eq!(sparse_arena.as_keys_slice(), &[8, 4, 6]);
        assert_eq!(sparse_arena.as_slice(), &[34, 69, 666]);

        sparse_arena.retain(|_, value| *value % 2 == 1);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.as_keys_slice(), &[4]);
        assert_eq!(sparse_arena.as_slice(), &[69]);
    }

    #[test]
    fn five_items_drain() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(8, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(4, 69);
        sparse_arena.insert(3, 228);
        sparse_arena.insert(6, 666);

        let drain = sparse_arena.drain();
        assert_eq!(drain.as_keys_slice(), &[8, 1, 4, 3, 6]);
        assert_eq!(drain.as_values_slice(), &[34, 42, 69, 228, 666]);

        forget(drain);
        assert_eq!(sparse_arena.len(), 0);
        assert_eq!(sparse_arena.sparse_len(), 0);
        assert_eq!(sparse_arena.keys().as_slice(), &[]);
        assert_eq!(sparse_arena.values().as_slice(), &[]);
    }

    #[test]
    fn five_items_insert_truncate() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(8, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(4, 69);
        sparse_arena.insert(3, 228);
        sparse_arena.insert(6, 666);

        sparse_arena.truncate(usize::MAX, 5);
        assert_eq!(sparse_arena.sparse_len(), 5);
        assert_eq!(sparse_arena.as_keys_slice(), &[1, 4, 3]);
        assert_eq!(sparse_arena.as_slice(), &[42, 69, 228]);

        assert_eq!(sparse_arena.get(1), Some(&42));
        assert_eq!(sparse_arena.get(4), Some(&69));
        assert_eq!(sparse_arena.get(3), Some(&228));

        sparse_arena.truncate(1, usize::MAX);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.as_keys_slice(), &[1]);
        assert_eq!(sparse_arena.as_slice(), &[42]);

        assert_eq!(sparse_arena.get(1), Some(&42));
    }

    #[test]
    fn five_items_push_truncate() {
        let mut sparse_arena = SparseArena::new();
        let key0 = sparse_arena.push(34);
        let key1 = sparse_arena.push(42);
        let key2 = sparse_arena.push(69);
        let key3 = sparse_arena.push(228);
        let key4 = sparse_arena.push(666);

        sparse_arena.truncate(usize::MAX, 3);
        assert_eq!(sparse_arena.sparse_len(), 3);
        assert_eq!(sparse_arena.as_keys_slice(), &[key0, key1, key2]);
        assert_eq!(sparse_arena.as_slice(), &[34, 42, 69]);

        assert_eq!(sparse_arena.get(key0), Some(&34));
        assert_eq!(sparse_arena.get(key1), Some(&42));
        assert_eq!(sparse_arena.get(key2), Some(&69));
        assert_eq!(sparse_arena.get(key3), None);
        assert_eq!(sparse_arena.get(key4), None);

        sparse_arena.truncate(1, usize::MAX);
        assert_eq!(sparse_arena.len(), 1);
        assert_eq!(sparse_arena.as_keys_slice(), &[key0]);
        assert_eq!(sparse_arena.as_slice(), &[34]);

        assert_eq!(sparse_arena.get(key0), Some(&34));
    }

    #[test]
    fn five_items_sort() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(8, 42);
        sparse_arena.insert(1, 228);
        sparse_arena.insert(4, 69);
        sparse_arena.insert(3, 666);
        sparse_arena.insert(6, 34);

        sparse_arena.sort();
        assert_eq!(sparse_arena.as_keys_slice(), &[6, 8, 4, 1, 3]);
        assert_eq!(sparse_arena.as_slice(), &[34, 42, 69, 228, 666]);

        assert_eq!(sparse_arena.get(8), Some(&42));
        assert_eq!(sparse_arena.get(1), Some(&228));
        assert_eq!(sparse_arena.get(4), Some(&69));
        assert_eq!(sparse_arena.get(3), Some(&666));
        assert_eq!(sparse_arena.get(6), Some(&34));
    }

    #[test]
    fn five_items_sort_keys() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(8, 42);
        sparse_arena.insert(1, 228);
        sparse_arena.insert(4, 69);
        sparse_arena.insert(3, 666);
        sparse_arena.insert(6, 34);

        sparse_arena.sort_keys();
        assert_eq!(sparse_arena.as_keys_slice(), &[1, 3, 4, 6, 8]);
        assert_eq!(sparse_arena.as_slice(), &[228, 666, 69, 34, 42]);

        assert_eq!(sparse_arena.get(8), Some(&42));
        assert_eq!(sparse_arena.get(1), Some(&228));
        assert_eq!(sparse_arena.get(4), Some(&69));
        assert_eq!(sparse_arena.get(3), Some(&666));
        assert_eq!(sparse_arena.get(6), Some(&34));
    }

    #[test]
    fn five_items_sort_by() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(8, 42);
        sparse_arena.insert(1, 228);
        sparse_arena.insert(4, 69);
        sparse_arena.insert(3, 666);
        sparse_arena.insert(6, 34);

        sparse_arena.sort_by(|(_, a), (_, b)| Ord::cmp(b, a));
        assert_eq!(sparse_arena.as_keys_slice(), &[3, 1, 4, 8, 6]);
        assert_eq!(sparse_arena.as_slice(), &[666, 228, 69, 42, 34]);

        assert_eq!(sparse_arena.get(8), Some(&42));
        assert_eq!(sparse_arena.get(1), Some(&228));
        assert_eq!(sparse_arena.get(4), Some(&69));
        assert_eq!(sparse_arena.get(3), Some(&666));
        assert_eq!(sparse_arena.get(6), Some(&34));
    }

    #[test]
    fn five_items_entry() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(8, 42);
        sparse_arena.insert(1, 228);
        sparse_arena.insert(4, 69);
        sparse_arena.insert(3, 666);
        sparse_arena.insert(6, 34);

        let entry = sparse_arena.entry(0);
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

        let sparse_arena: SparseArena<_> = keys.into_iter().zip(values).collect();
        assert_eq!(sparse_arena.len(), 4);
        assert_eq!(sparse_arena.keys().as_slice(), &[3, 10, 5, 1]);
        assert_eq!(sparse_arena.values().as_slice(), &[34, 228, 69, 666]);

        assert_eq!(sparse_arena.get(3), Some(&34));
        assert_eq!(sparse_arena.get(10), Some(&228));
        assert_eq!(sparse_arena.get(5), Some(&69));
        assert_eq!(sparse_arena.get(1), Some(&666));
    }

    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn from_keys_values_iter_too_large_key() {
        let keys = [3, 10, 5, 10, 1, usize::MAX];
        let values = [34, 42, 69, 228, 666, 999];

        let sparse_arena: SparseArena<_> = keys.into_iter().zip(values).collect();
        assert_eq!(sparse_arena.len(), 4);
        assert_eq!(sparse_arena.keys().as_slice(), &[3, 10, 5, 1, usize::MAX]);
        assert_eq!(sparse_arena.values().as_slice(), &[34, 228, 69, 666, 999]);

        assert_eq!(sparse_arena.get(3), Some(&34));
        assert_eq!(sparse_arena.get(10), Some(&228));
        assert_eq!(sparse_arena.get(5), Some(&69));
        assert_eq!(sparse_arena.get(1), Some(&666));
        assert_eq!(sparse_arena.get(usize::MAX), Some(&999));
    }

    #[test]
    fn from_values_iter() {
        let values = [34, 42, 69, 228, 666];
        let sparse_arena: SparseArena<_> = values.into_iter().collect();

        assert_eq!(sparse_arena.len(), 5);
        assert_eq!(sparse_arena.keys().as_slice(), &[0, 1, 2, 3, 4]);
        assert_eq!(sparse_arena.values().as_slice(), &[34, 42, 69, 228, 666]);

        assert_eq!(sparse_arena.get(0), Some(&34));
        assert_eq!(sparse_arena.get(1), Some(&42));
        assert_eq!(sparse_arena.get(2), Some(&69));
        assert_eq!(sparse_arena.get(3), Some(&228));
        assert_eq!(sparse_arena.get(4), Some(&666));
    }

    #[test]
    fn extend_keys_values() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(5, 69);

        let keys = [3, 0, 2, 8];
        let values = [228, 666, 42, 69];
        sparse_arena.extend(keys.into_iter().zip(values));

        assert_eq!(sparse_arena.keys().as_slice(), &[2, 1, 5, 3, 0, 8]);
        assert_eq!(
            sparse_arena.values().as_slice(),
            &[42, 42, 69, 228, 666, 69]
        );
    }

    #[test]
    fn extend_values() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, 34);
        sparse_arena.insert(1, 42);
        sparse_arena.insert(4, 69);

        let values = [228, 666, 201];
        sparse_arena.extend(values);

        assert_eq!(sparse_arena.keys().as_slice(), &[2, 1, 4, 3, 0, 5]);
        assert_eq!(
            sparse_arena.values().as_slice(),
            &[34, 42, 69, 228, 666, 201]
        );
    }
}
