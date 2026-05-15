use gpecs_itertools::Itertools;

use crate::{
    item::{ArenaSparseItem, SparseItem},
    key::{Epoch, Key, SparseIndex},
};

#[inline]
#[track_caller]
pub fn unwrap_sparse_item<T>(sparse: &[T], sparse_index: usize) -> &T {
    let Some(item) = sparse.get(sparse_index) else {
        assert_key_bounds_failed()
    };
    item
}

#[inline]
#[track_caller]
#[cfg_attr(not(feature = "alloc"), expect(dead_code))]
pub fn unwrap_sparse_item_mut<T>(sparse: &mut [T], sparse_index: usize) -> &mut T {
    let Some(item) = sparse.get_mut(sparse_index) else {
        assert_key_bounds_failed()
    };
    item
}

#[cold]
#[track_caller]
#[inline(never)]
const fn assert_occupied_failed() -> ! {
    panic!("current sparse item should be occupied")
}

#[inline]
#[track_caller]
pub fn assert_occupied(item: &impl SparseItem) {
    if item.is_occupied() {
        return;
    }
    assert_occupied_failed()
}

#[cold]
#[track_caller]
#[inline(never)]
const fn assert_vacant_failed() -> ! {
    panic!("current sparse item should be vacant")
}

#[inline]
#[track_caller]
pub fn assert_vacant(item: &impl SparseItem) {
    if item.is_vacant() {
        return;
    }
    assert_vacant_failed()
}

#[inline]
#[track_caller]
pub fn unwrap_dense_index<T>(item: &T) -> T::Index
where
    T: SparseItem,
{
    let Some(dense_index) = item.dense_index() else {
        assert_occupied_failed()
    };
    dense_index
}

#[inline]
#[track_caller]
pub fn unwrap_next_vacant<T>(item: &T) -> T::Index
where
    T: ArenaSparseItem,
{
    let Some(next_vacant) = item.next_vacant() else {
        assert_vacant_failed()
    };
    next_vacant
}

#[inline]
#[track_caller]
pub fn unwrap_dense<T>(dense: impl IntoIterator<Item = T>, dense_index: usize) -> T {
    let Some(item) = dense.into_iter().nth(dense_index) else {
        assert_dense_index_bounds_failed()
    };
    item
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_dense_pair_failed() -> ! {
    panic!("indices from sparse should be in bounds of dense and differ from each other")
}

#[inline]
#[track_caller]
pub fn unwrap_dense_pair<T>(
    iter: impl IntoIterator<Item = T>,
    first_index: usize,
    second_index: usize,
) -> (T, T) {
    let Some(pair) = iter.into_iter().get_pair(first_index, second_index) else {
        unwrap_dense_pair_failed()
    };
    pair
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_sparse_pair_failed() -> ! {
    panic!("keys should be in bounds of sparse and differ from each other")
}

#[inline]
#[track_caller]
pub fn unwrap_sparse_pair<T>(
    iter: impl IntoIterator<Item = T>,
    first_index: usize,
    second_index: usize,
) -> (T, T) {
    let Some(pair) = iter.into_iter().get_pair(first_index, second_index) else {
        unwrap_sparse_pair_failed()
    };
    pair
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_into_usize_failed() -> ! {
    panic!("index should be convertible to usize")
}

#[inline]
#[track_caller]
pub fn unwrap_into_usize<I>(index: I) -> usize
where
    I: TryInto<usize>,
{
    let Ok(index) = index.try_into() else {
        unwrap_into_usize_failed()
    };
    index
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_into_index_failed() -> ! {
    panic!("usize should be convertible to index")
}

#[inline]
#[track_caller]
pub fn unwrap_into_index<I>(index: usize) -> I
where
    usize: TryInto<I>,
{
    let Ok(index) = index.try_into() else {
        unwrap_into_index_failed()
    };
    index
}

#[inline]
#[track_caller]
pub fn unwrap_dense_from_sparse_index<K, V>(
    sparse_index: K::SparseIndex,
    dense: impl IntoIterator<Item = V>,
    sparse: &[impl SparseItem<Index = K::SparseIndex>],
) -> V
where
    K: Key,
{
    let sparse_index = unwrap_into_usize(sparse_index);
    let sparse_item = unwrap_sparse_item(sparse, sparse_index);
    let dense_index = unwrap_dense_index(sparse_item);
    let dense_index = unwrap_into_usize(dense_index);
    unwrap_dense(dense, dense_index)
}

#[cold]
#[track_caller]
#[inline(never)]
pub const fn assert_dense_index_bounds_failed() -> ! {
    panic!("index from sparse should be in bounds of dense")
}

#[inline]
#[track_caller]
pub const fn assert_dense_index_bounds(dense_index: usize, dense_len: usize) {
    if dense_index < dense_len {
        return;
    }
    assert_dense_index_bounds_failed()
}

#[cold]
#[track_caller]
#[inline(never)]
const fn assert_key_bounds_failed() -> ! {
    panic!("key from dense should be in bounds of sparse")
}

#[inline]
#[track_caller]
#[cfg_attr(not(feature = "alloc"), expect(dead_code))]
pub const fn assert_key_bounds(key: usize, sparse_len: usize) {
    if key < sparse_len {
        return;
    }
    assert_key_bounds_failed()
}

#[cold]
#[track_caller]
#[inline(never)]
const fn assert_equal_key_failed() -> ! {
    panic!("provided key and key from dense should be equal")
}

#[inline]
#[track_caller]
pub fn assert_equal_key<K>(key: K, dense_key: K)
where
    K: Key,
{
    if key == dense_key {
        return;
    }
    assert_equal_key_failed()
}

#[inline]
#[track_caller]
pub fn assert_compatible_key<K>(key: K, dense_key: K)
where
    K: Key,
{
    assert_equal_sparse_index(key.sparse_index(), dense_key.sparse_index());
    assert_equal_epoch(key.epoch(), dense_key.epoch());
}

#[cold]
#[track_caller]
#[inline(never)]
const fn assert_equal_epoch_failed() -> ! {
    panic!("provided epoch does not match an actual epoch")
}

#[inline]
#[track_caller]
pub fn assert_equal_epoch<E>(first: E, second: E)
where
    E: Epoch,
{
    if first == second {
        return;
    }
    assert_equal_epoch_failed()
}

#[cold]
#[track_caller]
#[inline(never)]
const fn assert_equal_sparse_index_failed() -> ! {
    panic!("provided sparse index does not match an actual sparse index")
}

#[inline]
#[track_caller]
pub fn assert_equal_sparse_index<I>(first: I, second: I)
where
    I: SparseIndex,
{
    if first == second {
        return;
    }
    assert_equal_sparse_index_failed()
}
