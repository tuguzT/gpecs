use core::{fmt::Display, mem::swap};

use crate::{
    assert::{
        check_dense_index_bounds, check_equal_key, unwrap_dense_key, unwrap_dense_value,
        unwrap_dense_value_mut, unwrap_dense_value_pair_mut,
    },
    item::{SparseItem, SparseItemKind},
    key::Key,
};

pub fn get_pair_mut<T>(slice: &mut [T], a: usize, b: usize) -> Option<(&mut T, &mut T)> {
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

pub fn sparse_swap<K, V>(
    dense: &mut [V],
    sparse: &mut [SparseItem<K::Epoch>],
    first_key: K,
    second_key: K,
) where
    K: Key,
{
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

    let (first_value, second_value) = unwrap_dense_value_pair_mut(dense, first_index, second_index);
    swap(first_value, second_value);
}

pub fn sparse_swap_keys<K>(
    dense: &mut [K],
    sparse: &mut [SparseItem<K::Epoch>],
    first_key: K,
    second_key: K,
) where
    K: Key,
{
    let first_index = first_key.sparse_index();
    let second_index = second_key.sparse_index();
    let Some((first_item, second_item)) = get_pair_mut(sparse, first_index, second_index) else {
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

    let (first_key, second_key) = unwrap_dense_value_pair_mut(dense, first_index, second_index);
    swap(first_item, second_item);
    swap(first_key, second_key);
}

pub fn sparse_get<'a, K, V>(
    dense_keys: &[K],
    dense_values: &'a [V],
    sparse: &[SparseItem<K::Epoch>],
    key: K,
) -> Option<&'a V>
where
    K: Key,
{
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

#[cold]
#[track_caller]
#[inline(never)]
fn sparse_index_failed<K>(key: &K) -> !
where
    K: Display,
{
    panic!("key {key} not found")
}

#[inline]
#[track_caller]
pub fn sparse_index<'a, K, V>(
    dense_keys: &[K],
    dense_values: &'a [V],
    sparse: &[SparseItem<K::Epoch>],
    key: K,
) -> &'a V
where
    K: Key + Display,
{
    match sparse_get(dense_keys, dense_values, sparse, key) {
        Some(value) => value,
        None => sparse_index_failed(&key),
    }
}

pub fn sparse_get_mut<'a, K, V>(
    dense_keys: &[K],
    dense_values: &'a mut [V],
    sparse: &[SparseItem<K::Epoch>],
    key: K,
) -> Option<&'a mut V>
where
    K: Key,
{
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

#[inline]
#[track_caller]
pub fn sparse_index_mut<'a, K, V>(
    dense_keys: &[K],
    dense_values: &'a mut [V],
    sparse: &[SparseItem<K::Epoch>],
    key: K,
) -> &'a mut V
where
    K: Key + Display,
{
    match sparse_get_mut(dense_keys, dense_values, sparse, key) {
        Some(value) => value,
        None => sparse_index_failed(&key),
    }
}

pub fn sparse_get_with_key<'a, K, V>(
    dense_keys: &[K],
    dense_values: &'a [V],
    sparse: &[SparseItem<K::Epoch>],
    sparse_index: usize,
) -> Option<(K, &'a V)>
where
    K: Key,
{
    let sparse_item = sparse.get(sparse_index)?;
    let dense_index = sparse_item.dense_index()?;

    let value = unwrap_dense_value(dense_values, dense_index);
    let key = *unwrap_dense_key(dense_keys, dense_index);
    check_equal_key(key, K::new(sparse_index, sparse_item.epoch));

    Some((key, value))
}

pub fn sparse_get_mut_with_key<'a, K, V>(
    dense_keys: &[K],
    dense_values: &'a mut [V],
    sparse: &[SparseItem<K::Epoch>],
    sparse_index: usize,
) -> Option<(K, &'a mut V)>
where
    K: Key,
{
    let sparse_item = sparse.get(sparse_index)?;
    let dense_index = sparse_item.dense_index()?;

    let value = unwrap_dense_value_mut(dense_values, dense_index);
    let key = *unwrap_dense_key(dense_keys, dense_index);
    check_equal_key(key, K::new(sparse_index, sparse_item.epoch));

    Some((key, value))
}

pub fn sparse_get_epoch<K>(
    dense_keys: &[K],
    sparse: &[SparseItem<K::Epoch>],
    sparse_index: usize,
) -> Option<K::Epoch>
where
    K: Key,
{
    let sparse_item = sparse.get(sparse_index)?;
    let epoch = sparse_item.epoch;
    if let Some(dense_index) = sparse_item.dense_index() {
        let key = *unwrap_dense_key(dense_keys, dense_index);
        check_equal_key(key, K::new(sparse_index, epoch));
    }

    Some(epoch)
}

pub fn sparse_contains_key<K>(dense_keys: &[K], sparse: &[SparseItem<K::Epoch>], key: K) -> bool
where
    K: Key,
{
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
