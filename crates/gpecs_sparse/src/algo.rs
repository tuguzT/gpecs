use core::{borrow::Borrow, fmt::Debug, ops};

use crate::{
    assert::{
        check_compatible_key, check_dense_index_bounds, check_equal_key, unwrap_dense,
        unwrap_into_usize,
    },
    item::{SparseItem, SparseItemKind},
    key::Key,
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

#[inline]
fn sparse_item_by_key<K>(sparse: &[SparseItem<K>], key: K) -> Option<&SparseItem<K>>
where
    K: Key,
{
    let sparse_index = key.sparse_index().try_into().ok()?;
    let sparse_item = sparse
        .get(sparse_index)
        .take_if(|item| item.epoch == key.epoch())?;
    Some(sparse_item)
}

pub fn sparse_get<K, T>(
    dense: impl IntoIterator<Item = (impl Borrow<K>, T)>,
    sparse: &[SparseItem<K>],
    key: K,
) -> Option<T>
where
    K: Key,
{
    let sparse_item = sparse_item_by_key(sparse, key)?;
    let dense_index = unwrap_into_usize(sparse_item.into_dense_index()?);

    let (dense_key, value) = unwrap_dense(dense, dense_index);
    let &dense_key = dense_key.borrow();
    check_equal_key(key, dense_key);

    Some(value)
}

#[cold]
#[track_caller]
#[inline(never)]
fn sparse_index_failed<K>(key: &K) -> !
where
    K: Debug,
{
    panic!("key {key:?} not found")
}

#[inline]
#[track_caller]
pub fn sparse_index<K, T>(
    dense: impl IntoIterator<Item = (impl Borrow<K>, T)>,
    sparse: &[SparseItem<K>],
    key: K,
) -> T
where
    K: Key + Debug,
{
    match sparse_get(dense, sparse, key) {
        Some(value) => value,
        None => sparse_index_failed(&key),
    }
}

#[inline]
fn sparse_item<K>(sparse: &[SparseItem<K>], sparse_index: K::SparseIndex) -> Option<&SparseItem<K>>
where
    K: Key,
{
    let sparse_index = sparse_index.try_into().ok()?;
    let sparse_item = sparse.get(sparse_index)?;
    Some(sparse_item)
}

pub fn sparse_get_with_key<K, T>(
    dense: impl IntoIterator<Item = (impl Borrow<K>, T)>,
    sparse: &[SparseItem<K>],
    sparse_index: K::SparseIndex,
) -> Option<(K, T)>
where
    K: Key,
{
    let sparse_item = sparse_item(sparse, sparse_index)?;
    let dense_index = unwrap_into_usize(sparse_item.into_dense_index()?);

    let (dense_key, value) = unwrap_dense(dense, dense_index);
    let &dense_key = dense_key.borrow();
    check_compatible_key(K::new(sparse_index, sparse_item.epoch), dense_key);

    Some((dense_key, value))
}

pub fn sparse_get_epoch<K>(
    dense_keys: &[K],
    sparse: &[SparseItem<K>],
    sparse_index: K::SparseIndex,
) -> Option<K::Epoch>
where
    K: Key,
{
    let sparse_item = sparse_item(sparse, sparse_index)?;
    let epoch = sparse_item.epoch;

    if let Some(dense_index) = sparse_item.dense_index() {
        let dense_index = unwrap_into_usize(*dense_index);
        let &dense_key = unwrap_dense(dense_keys, dense_index);
        check_compatible_key(K::new(sparse_index, epoch), dense_key);
    }

    Some(epoch)
}

pub fn sparse_contains_key<K>(dense_keys: &[K], sparse: &[SparseItem<K>], key: K) -> bool
where
    K: Key,
{
    let Some(sparse_item) = sparse_item_by_key(sparse, key) else {
        return false;
    };
    let SparseItemKind::Occupied { dense_index } = sparse_item.kind else {
        return false;
    };
    let dense_index = unwrap_into_usize(dense_index);

    check_dense_index_bounds(dense_index, dense_keys.len());
    true
}
