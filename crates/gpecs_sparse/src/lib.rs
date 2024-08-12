//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod arena;
pub mod key;
pub mod prelude;
pub mod set;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SparseItem<E> {
    pub kind: SparseItemKind,
    pub epoch: E,
}

impl<E> SparseItem<E> {
    #[inline]
    pub const fn new(kind: SparseItemKind, epoch: E) -> Self {
        Self { kind, epoch }
    }

    #[inline]
    pub const fn occupied(dense_index: usize, epoch: E) -> Self {
        let kind = SparseItemKind::occupied(dense_index);
        Self::new(kind, epoch)
    }

    #[inline]
    pub const fn vacant(next_vacant: usize, epoch: E) -> Self {
        let kind = SparseItemKind::vacant(next_vacant);
        Self::new(kind, epoch)
    }

    #[inline]
    pub const fn is_occupied(&self) -> bool {
        let Self { kind, .. } = self;
        kind.is_occupied()
    }

    #[inline]
    pub const fn is_vacant(&self) -> bool {
        let Self { kind, .. } = self;
        kind.is_vacant()
    }

    #[inline]
    pub const fn kind(&self) -> &SparseItemKind {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub fn kind_mut(&mut self) -> &mut SparseItemKind {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub const fn dense_index(&self) -> Option<usize> {
        let Self { kind, .. } = self;
        kind.dense_index()
    }

    #[inline]
    pub fn dense_index_mut(&mut self) -> Option<&mut usize> {
        let Self { kind, .. } = self;
        kind.dense_index_mut()
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<usize> {
        let Self { kind, .. } = self;
        kind.next_vacant()
    }

    #[inline]
    pub fn next_vacant_mut(&mut self) -> Option<&mut usize> {
        let Self { kind, .. } = self;
        kind.next_vacant_mut()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum SparseItemKind {
    Occupied { dense_index: usize },
    Vacant { next_vacant: usize },
}

impl SparseItemKind {
    #[inline]
    pub const fn occupied(dense_index: usize) -> Self {
        Self::Occupied { dense_index }
    }

    #[inline]
    pub const fn vacant(next_vacant: usize) -> Self {
        Self::Vacant { next_vacant }
    }

    #[inline]
    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied { .. })
    }

    #[inline]
    pub const fn is_vacant(&self) -> bool {
        matches!(self, Self::Vacant { .. })
    }

    #[inline]
    pub const fn dense_index(&self) -> Option<usize> {
        match self {
            Self::Occupied { dense_index } => Some(*dense_index),
            Self::Vacant { .. } => None,
        }
    }

    #[inline]
    pub fn dense_index_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { dense_index } => Some(dense_index),
            Self::Vacant { .. } => None,
        }
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<usize> {
        match self {
            Self::Occupied { .. } => None,
            Self::Vacant { next_vacant } => Some(*next_vacant),
        }
    }

    #[inline]
    pub fn next_vacant_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { .. } => None,
            Self::Vacant { next_vacant } => Some(next_vacant),
        }
    }
}

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

#[cold]
#[track_caller]
#[inline(never)]
const fn check_kv_same_len_failed() -> ! {
    panic!("keys and values should have the same length")
}

#[cold]
#[track_caller]
#[inline(never)]
const fn check_kv_same_capacity_failed() -> ! {
    panic!("keys and values should have the same capacity")
}

#[inline]
#[track_caller]
fn match_kv_same_kind<K, V>(key: Option<K>, value: Option<V>) -> Option<(K, V)> {
    match (key, value) {
        (Some(key), Some(value)) => Some((key, value)),
        (None, None) => None,
        _ => check_kv_same_len_failed(),
    }
}

#[inline]
#[track_caller]
fn unwrap_sparse_item<E>(sparse: &[SparseItem<E>], sparse_index: usize) -> &SparseItem<E> {
    let Some(item) = sparse.get(sparse_index) else {
        check_key_bounds_failed()
    };
    item
}

#[inline]
#[track_caller]
fn unwrap_sparse_item_mut<E>(sparse: &mut [SparseItem<E>], key: usize) -> &mut SparseItem<E> {
    let Some(item) = sparse.get_mut(key) else {
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
fn unwrap_dense_index(kind: &SparseItemKind) -> usize {
    let Some(dense_index) = kind.dense_index() else {
        unwrap_dense_index_failed()
    };
    dense_index
}

#[inline]
#[track_caller]
fn unwrap_dense_index_mut(kind: &mut SparseItemKind) -> &mut usize {
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
fn unwrap_next_vacant(kind: &SparseItemKind) -> usize {
    let Some(next_vacant) = kind.next_vacant() else {
        unwrap_next_vacant_failed()
    };
    next_vacant
}

#[inline]
#[track_caller]
fn unwrap_dense_key<K>(keys: &[K], dense_index: usize) -> &K {
    let Some(dense_key) = keys.get(dense_index) else {
        check_dense_index_bounds_failed();
    };
    dense_key
}

#[inline]
#[track_caller]
fn unwrap_dense_value<T>(values: &[T], dense_index: usize) -> &T {
    let Some(dense_value) = values.get(dense_index) else {
        check_dense_index_bounds_failed();
    };
    dense_value
}

#[inline]
#[track_caller]
fn unwrap_dense_value_mut<T>(values: &mut [T], dense_index: usize) -> &mut T {
    let Some(dense_value) = values.get_mut(dense_index) else {
        check_dense_index_bounds_failed();
    };
    dense_value
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_dense_value_pair_mut_failed() -> ! {
    panic!("indices from sparse should be in bounds of dense and differ from each other")
}

#[inline]
#[track_caller]
fn unwrap_dense_value_pair_mut<T>(
    values: &mut [T],
    first_index: usize,
    second_index: usize,
) -> (&mut T, &mut T) {
    let Some(pair) = get_pair_mut(values, first_index, second_index) else {
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
fn unwrap_sparse_items_pair_mut<E>(
    sparse: &mut [SparseItem<E>],
    first_index: usize,
    second_index: usize,
) -> (&mut SparseItem<E>, &mut SparseItem<E>) {
    let Some(pair) = get_pair_mut(sparse, first_index, second_index) else {
        unwrap_sparse_items_pair_mut_failed()
    };
    pair
}

#[inline]
#[track_caller]
fn unwrap_value_from_sparse_index<'a, T, E>(
    sparse_index: usize,
    values: &'a [T],
    sparse: &[SparseItem<E>],
) -> &'a T {
    let sparse_item = unwrap_sparse_item(sparse, sparse_index);
    let dense_index = unwrap_dense_index(&sparse_item.kind);
    unwrap_dense_value(values, dense_index)
}

#[cold]
#[track_caller]
#[inline(never)]
const fn check_dense_index_bounds_failed() -> ! {
    panic!("index from sparse should be in bounds of dense")
}

#[inline]
#[track_caller]
fn check_dense_index_bounds(dense_index: usize, dense_len: usize) {
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
fn check_key_bounds(key: usize, sparse_len: usize) {
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
fn check_equal_key<K>(key: K, dense_key: K)
where
    K: PartialEq,
{
    if key == dense_key {
        return;
    }
    check_equal_key_failed()
}

#[inline]
#[track_caller]
fn check_kv_same_len(keys_len: usize, values_len: usize) {
    if keys_len == values_len {
        return;
    }
    check_kv_same_len_failed()
}

#[inline]
#[track_caller]
fn check_kv_same_capacity(keys_capacity: usize, values_capacity: usize) {
    if keys_capacity == values_capacity {
        return;
    }
    check_kv_same_capacity_failed()
}
