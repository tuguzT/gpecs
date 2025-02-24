use crate::{
    algo::get_pair,
    item::{SparseItem, SparseItemKind},
};

#[inline]
#[track_caller]
pub fn unwrap_sparse_item<E>(sparse: &[SparseItem<E>], sparse_index: usize) -> &SparseItem<E> {
    let Some(item) = sparse.get(sparse_index) else {
        check_key_bounds_failed()
    };
    item
}

#[inline]
#[track_caller]
pub fn unwrap_sparse_item_mut<E>(
    sparse: &mut [SparseItem<E>],
    sparse_index: usize,
) -> &mut SparseItem<E> {
    let Some(item) = sparse.get_mut(sparse_index) else {
        check_key_bounds_failed()
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
pub const fn unwrap_dense_index(kind: &SparseItemKind) -> usize {
    let Some(dense_index) = kind.dense_index() else {
        unwrap_dense_index_failed()
    };
    dense_index
}

#[inline]
#[track_caller]
pub fn unwrap_dense_index_mut(kind: &mut SparseItemKind) -> &mut usize {
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
pub const fn unwrap_next_vacant(kind: &SparseItemKind) -> usize {
    let Some(next_vacant) = kind.next_vacant() else {
        unwrap_next_vacant_failed()
    };
    next_vacant
}

#[inline]
#[track_caller]
pub const fn unwrap_next_vacant_mut(kind: &mut SparseItemKind) -> &mut usize {
    let Some(next_vacant) = kind.next_vacant_mut() else {
        unwrap_next_vacant_failed()
    };
    next_vacant
}

#[inline]
#[track_caller]
pub fn unwrap_dense<T>(dense: impl IntoIterator<Item = T>, dense_index: usize) -> T {
    let Some(item) = dense.into_iter().nth(dense_index) else {
        check_dense_index_bounds_failed()
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
pub fn unwrap_sparse_items_pair_mut<E>(
    sparse: &mut [SparseItem<E>],
    first_index: usize,
    second_index: usize,
) -> (&mut SparseItem<E>, &mut SparseItem<E>) {
    let Some(pair) = get_pair(sparse, first_index, second_index) else {
        unwrap_sparse_items_pair_mut_failed()
    };
    pair
}

// TODO uncomment this when issues of `view` module will be resolved
// #[inline]
// #[track_caller]
// pub fn unwrap_dense_from_sparse_index<T, E>(
//     sparse_index: usize,
//     dense: impl IntoIterator<Item = T>,
//     sparse: &[SparseItem<E>],
// ) -> T {
//     let sparse_item = unwrap_sparse_item(sparse, sparse_index);
//     let dense_index = unwrap_dense_index(&sparse_item.kind);
//     unwrap_dense(dense, dense_index)
// }

#[cold]
#[track_caller]
#[inline(never)]
pub const fn check_dense_index_bounds_failed() -> ! {
    panic!("index from sparse should be in bounds of dense")
}

#[inline]
#[track_caller]
pub const fn check_dense_index_bounds(dense_index: usize, dense_len: usize) {
    if dense_index < dense_len {
        return;
    }
    check_dense_index_bounds_failed()
}

#[cold]
#[track_caller]
#[inline(never)]
const fn check_key_bounds_failed() -> ! {
    panic!("key from dense should be in bounds of sparse")
}

#[inline]
#[track_caller]
pub const fn check_key_bounds(key: usize, sparse_len: usize) {
    if key < sparse_len {
        return;
    }
    check_key_bounds_failed()
}

#[cold]
#[track_caller]
#[inline(never)]
const fn check_equal_key_failed() -> ! {
    panic!("provided key and key from dense should be equal")
}

#[inline]
#[track_caller]
pub fn check_equal_key<K>(key: K, dense_key: K)
where
    K: PartialEq,
{
    if key == dense_key {
        return;
    }
    check_equal_key_failed()
}
