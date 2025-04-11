use crate::{error::TooLargeSparseIndexError, key::Key};

use super::error::TryModifyErrorKind;

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
    use TryModifyErrorKind::*;

    match error {
        TooLargeSparseIndex(_) => panic!("{message} (sparse index is too large for `usize`)"),
        TooSmallSparseIndex(_) => panic!("{message} (sparse index is too small for `usize`)"),
        TryReserve(error) => panic!("{message} ({error})"),
    }
}
