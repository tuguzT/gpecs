use core::{fmt::Display, mem::swap, ops};

use gpecs_soa::slice::SoaSlicesMut;

use crate::{
    assert::{check_dense_index_bounds, check_equal_key, unwrap_dense, unwrap_dense_pair},
    item::{SparseItem, SparseItemKind},
    key::Key,
    pair::{KeyValueRefs, KeyValueRefsMut},
    soa::{mem::swap as soa_swap, traits::Soa},
};

// https://stackoverflow.com/a/73428605/14928295
fn get_range<T>(
    iterator: impl IntoIterator<Item = T>,
    range: impl ops::RangeBounds<usize>,
) -> impl Iterator<Item = T> {
    let start_bound = match range.start_bound() {
        ops::Bound::Included(&num) => num,
        ops::Bound::Excluded(&num) => num + 1,
        ops::Bound::Unbounded => 0,
    };

    let mut end_bound = match range.end_bound() {
        ops::Bound::Included(&num) => Some(num + 1),
        ops::Bound::Excluded(&num) => Some(num),
        ops::Bound::Unbounded => None,
    };

    iterator
        .into_iter()
        .take_while(move |_| {
            if let Some(num) = &mut end_bound {
                if *num == 0 {
                    false
                } else {
                    *num -= 1;
                    true
                }
            } else {
                true
            }
        })
        .skip(start_bound)
}

pub fn get_pair<T>(iter: impl IntoIterator<Item = T>, a: usize, b: usize) -> Option<(T, T)> {
    let (first, second) = (usize::min(a, b), usize::max(a, b));

    let mut iter = get_range(iter, first..=second);
    let first = iter.next()?;
    let second = iter.last()?;

    let pair = if a < b {
        (first, second)
    } else {
        (second, first)
    };
    Some(pair)
}

pub fn sparse_swap<'a, K, V>(
    context: &V::Context,
    dense: SoaSlicesMut<'a, V>,
    sparse: &mut [SparseItem<K::Epoch>],
    first_key: K,
    second_key: K,
) where
    K: Key,
    V: Soa + 'a,
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

    let (first_value, second_value) = unwrap_dense_pair(dense, first_index, second_index);
    soa_swap::<V>(context, first_value, second_value);
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
    let Some((first_item, second_item)) = get_pair(sparse, first_index, second_index) else {
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

    let (first_key, second_key) = unwrap_dense_pair(dense, first_index, second_index);
    swap(first_item, second_item);
    swap(first_key, second_key);
}

pub fn sparse_get<'a, K, V>(
    dense: impl IntoIterator<Item = KeyValueRefs<'a, K, V>>,
    sparse: &[SparseItem<K::Epoch>],
    key: K,
) -> Option<V::Refs<'a>>
where
    K: Key + 'a,
    V: Soa,
{
    let sparse_index = key.sparse_index();
    let sparse_item = sparse
        .get(sparse_index)
        .take_if(|item| item.epoch == key.epoch())?;
    let dense_index = sparse_item.dense_index()?;

    let (&dense_key, value) = unwrap_dense(dense, dense_index).into();
    check_equal_key(key, dense_key);

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
    dense: impl IntoIterator<Item = KeyValueRefs<'a, K, V>>,
    sparse: &[SparseItem<K::Epoch>],
    key: K,
) -> V::Refs<'a>
where
    K: Key + Display + 'a,
    V: Soa,
{
    match sparse_get(dense, sparse, key) {
        Some(value) => value,
        None => sparse_index_failed(&key),
    }
}

pub fn sparse_get_mut<'a, K, V>(
    dense: impl IntoIterator<Item = KeyValueRefsMut<'a, K, V>>,
    sparse: &[SparseItem<K::Epoch>],
    key: K,
) -> Option<V::RefsMut<'a>>
where
    K: Key + 'a,
    V: Soa,
{
    let sparse_index = key.sparse_index();
    let sparse_item = sparse
        .get(sparse_index)
        .take_if(|item| item.epoch == key.epoch())?;
    let dense_index = sparse_item.dense_index()?;

    let (&mut dense_key, value) = unwrap_dense(dense, dense_index).into();
    check_equal_key(key, dense_key);

    Some(value)
}

#[inline]
#[track_caller]
pub fn sparse_index_mut<'a, K, V>(
    dense: impl IntoIterator<Item = KeyValueRefsMut<'a, K, V>>,
    sparse: &[SparseItem<K::Epoch>],
    key: K,
) -> V::RefsMut<'a>
where
    K: Key + Display + 'a,
    V: Soa,
{
    match sparse_get_mut(dense, sparse, key) {
        Some(value) => value,
        None => sparse_index_failed(&key),
    }
}

pub fn sparse_get_with_key<'a, K, V>(
    dense: impl IntoIterator<Item = KeyValueRefs<'a, K, V>>,
    sparse: &[SparseItem<K::Epoch>],
    sparse_index: usize,
) -> Option<(K, V::Refs<'a>)>
where
    K: Key + 'a,
    V: Soa,
{
    let sparse_item = sparse.get(sparse_index)?;
    let dense_index = sparse_item.dense_index()?;

    let (&key, value) = unwrap_dense(dense, dense_index).into();
    check_equal_key(key, K::new(sparse_index, sparse_item.epoch));

    Some((key, value))
}

pub fn sparse_get_mut_with_key<'a, K, V>(
    dense: impl IntoIterator<Item = KeyValueRefsMut<'a, K, V>>,
    sparse: &[SparseItem<K::Epoch>],
    sparse_index: usize,
) -> Option<(K, V::RefsMut<'a>)>
where
    K: Key + 'a,
    V: Soa,
{
    let sparse_item = sparse.get(sparse_index)?;
    let dense_index = sparse_item.dense_index()?;

    let (&mut key, value) = unwrap_dense(dense, dense_index).into();
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
        let key = *unwrap_dense(dense_keys, dense_index);
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
