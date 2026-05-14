use crate::{error::TooLargeSparseIndexError, item::DefaultSparseItem, key::Key};

use super::error::TryModifyErrorKind;

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_next_vacant_failed() -> ! {
    panic!("current sparse item should be vacant")
}

#[inline]
#[track_caller]
pub const fn unwrap_next_vacant<K>(item: &DefaultSparseItem<K>) -> &K::SparseIndex
where
    K: Key,
{
    let Some(next_vacant) = item.next_vacant() else {
        unwrap_next_vacant_failed()
    };
    next_vacant
}

#[inline]
#[track_caller]
pub const fn unwrap_next_vacant_mut<K>(item: &mut DefaultSparseItem<K>) -> &mut K::SparseIndex
where
    K: Key,
{
    let Some(next_vacant) = item.next_vacant_mut() else {
        unwrap_next_vacant_failed()
    };
    next_vacant
}

#[cold]
#[inline(never)]
#[track_caller]
pub fn try_insert_failed<K>(error: TryModifyErrorKind<K>) -> !
where
    K: Key,
{
    try_modify_failed(error, "failed to insert value by provided key")
}

#[cold]
#[inline(never)]
#[track_caller]
pub fn try_push_failed<K>(error: TryModifyErrorKind<K>) -> !
where
    K: Key,
{
    try_modify_failed(error, "failed to push value")
}

#[cold]
#[inline(never)]
#[track_caller]
pub fn try_entry_failed<K>(_: TooLargeSparseIndexError<K>) -> !
where
    K: Key,
{
    panic!("failed to create entry for provided key (sparse index is too large for `usize`)")
}

#[cold]
#[inline(never)]
#[track_caller]
pub fn try_replace_key_failed<K>(error: TryModifyErrorKind<K>) -> !
where
    K: Key,
{
    try_modify_failed(error, "failed to replace key")
}

#[inline]
#[track_caller]
fn try_modify_failed<K>(error: TryModifyErrorKind<K>, message: &str) -> !
where
    K: Key,
{
    use TryModifyErrorKind::{TooLargeSparseIndex, TooSmallSparseIndex, TryReserve};

    match error {
        TooLargeSparseIndex(_) => panic!("{message} (sparse index is too large for `usize`)"),
        TooSmallSparseIndex(_) => panic!("{message} (sparse index is too small for `usize`)"),
        TryReserve(error) => panic!("{message} ({error})"),
    }
}
