use crate::{
    algo::get_pair,
    item::{SparseItem, SparseItemKind},
    key::{Epoch, Key},
};

#[inline]
#[track_caller]
pub fn unwrap_sparse_item<K>(sparse: &[SparseItem<K>], sparse_index: usize) -> &SparseItem<K>
where
    K: Key,
{
    let Some(item) = sparse.get(sparse_index) else {
        assert_key_bounds_failed()
    };
    item
}

#[inline]
#[track_caller]
#[cfg_attr(not(feature = "alloc"), expect(dead_code))]
pub fn unwrap_sparse_item_mut<K>(
    sparse: &mut [SparseItem<K>],
    sparse_index: usize,
) -> &mut SparseItem<K>
where
    K: Key,
{
    let Some(item) = sparse.get_mut(sparse_index) else {
        assert_key_bounds_failed()
    };
    item
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_dense_index_failed() -> ! {
    panic!("current sparse item should be occupied")
}

#[inline]
#[track_caller]
pub const fn unwrap_dense_index<I>(kind: &SparseItemKind<I>) -> &I {
    let Some(dense_index) = kind.dense_index() else {
        unwrap_dense_index_failed()
    };
    dense_index
}

#[inline]
#[track_caller]
pub fn unwrap_dense_index_mut<I>(kind: &mut SparseItemKind<I>) -> &mut I {
    let Some(dense_index) = kind.dense_index_mut() else {
        unwrap_dense_index_failed()
    };
    dense_index
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_next_vacant_failed() -> ! {
    panic!("current sparse item should be vacant")
}

#[inline]
#[track_caller]
#[cfg_attr(not(feature = "alloc"), expect(dead_code))]
pub const fn unwrap_next_vacant<I>(kind: &SparseItemKind<I>) -> &I {
    let Some(next_vacant) = kind.next_vacant() else {
        unwrap_next_vacant_failed()
    };
    next_vacant
}

#[inline]
#[track_caller]
#[cfg_attr(not(feature = "alloc"), expect(dead_code))]
pub const fn unwrap_next_vacant_mut<I>(kind: &mut SparseItemKind<I>) -> &mut I {
    let Some(next_vacant) = kind.next_vacant_mut() else {
        unwrap_next_vacant_failed()
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
const fn unwrap_dense_value_pair_mut_failed() -> ! {
    panic!("indices from sparse should be in bounds of dense and differ from each other")
}

#[inline]
#[track_caller]
pub fn unwrap_dense_pair<T>(
    iter: impl IntoIterator<Item = T>,
    first_index: usize,
    second_index: usize,
) -> (T, T) {
    let Some(pair) = get_pair(iter, first_index, second_index) else {
        unwrap_dense_value_pair_mut_failed()
    };
    pair
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_sparse_items_pair_mut_failed() -> ! {
    panic!("keys should be in bounds of sparse and differ from each other")
}

#[inline]
#[track_caller]
pub fn unwrap_sparse_items_pair_mut<K>(
    sparse: &mut [SparseItem<K>],
    first_index: usize,
    second_index: usize,
) -> (&mut SparseItem<K>, &mut SparseItem<K>)
where
    K: Key,
{
    let Some(pair) = get_pair(sparse, first_index, second_index) else {
        unwrap_sparse_items_pair_mut_failed()
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
    sparse: &[SparseItem<K>],
) -> V
where
    K: Key,
{
    let sparse_index = unwrap_into_usize(sparse_index);
    let sparse_item = unwrap_sparse_item(sparse, sparse_index);
    let dense_index = unwrap_dense_index(&sparse_item.kind);
    let dense_index = unwrap_into_usize(*dense_index);
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

#[cold]
#[track_caller]
#[inline(never)]
const fn assert_compatible_key_failed() -> ! {
    panic!("provided key and key from dense should have the same sparse index")
}

#[inline]
#[track_caller]
pub fn assert_compatible_key<K>(key: K, dense_key: K)
where
    K: Key,
{
    assert_equal_epoch(key.epoch(), dense_key.epoch());
    if key.sparse_index() == dense_key.sparse_index() {
        return;
    }
    assert_compatible_key_failed()
}

#[cold]
#[track_caller]
#[inline(never)]
const fn assert_equal_epoch_failed() -> ! {
    panic!("epoch provided by key does not match an actual epoch")
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
