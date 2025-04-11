use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use core_alloc::collections::TryReserveError as AllocTryReserveError;

use crate::{
    error::{TooLargeSparseIndexError, TooSmallSparseIndexError},
    key::Key,
    soa::vec::TryReserveError as SoaTryReserveError,
};

#[derive(Clone, PartialEq, Eq)]
pub enum TryReserveError {
    Sparse(AllocTryReserveError),
    Dense(SoaTryReserveError),
}

impl From<AllocTryReserveError> for TryReserveError {
    fn from(value: AllocTryReserveError) -> Self {
        Self::Sparse(value)
    }
}

impl From<SoaTryReserveError> for TryReserveError {
    fn from(value: SoaTryReserveError) -> Self {
        Self::Dense(value)
    }
}

impl Debug for TryReserveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        match self {
            Self::Sparse(error) => f.debug_tuple("Sparse").field(error).finish(),
            Self::Dense(error) => f.debug_tuple("Dense").field(error).finish(),
        }
    }
}

impl Display for TryReserveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sparse(error) => write!(f, "sparse: {error}"),
            Self::Dense(error) => write!(f, "dense: {error}"),
        }
    }
}

impl Error for TryReserveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sparse(error) => Some(error),
            Self::Dense(error) => Some(error),
        }
    }
}

pub enum TryModifyError<K>
where
    K: Key,
{
    TooLargeSparseIndex(TooLargeSparseIndexError<K>),
    TooSmallSparseIndex(TooSmallSparseIndexError<K>),
    TryReserve(TryReserveError),
}

impl<K> Debug for TryModifyError<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Debug,
    TooSmallSparseIndexError<K>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLargeSparseIndex(error) => {
                f.debug_tuple("TooLargeSparseIndex").field(error).finish()
            }
            Self::TooSmallSparseIndex(error) => {
                f.debug_tuple("TooSmallSparseIndex").field(error).finish()
            }
            Self::TryReserve(error) => f.debug_tuple("TryReserve").field(error).finish(),
        }
    }
}

impl<K> Display for TryModifyError<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Display,
    TooSmallSparseIndexError<K>: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLargeSparseIndex(error) => Display::fmt(error, f),
            Self::TooSmallSparseIndex(error) => Display::fmt(error, f),
            Self::TryReserve(error) => Display::fmt(error, f),
        }
    }
}

impl<K> Error for TryModifyError<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Error,
    TooSmallSparseIndexError<K>: Error,
{
}

impl<K> From<TooLargeSparseIndexError<K>> for TryModifyError<K>
where
    K: Key,
{
    fn from(value: TooLargeSparseIndexError<K>) -> Self {
        Self::TooLargeSparseIndex(value)
    }
}

impl<K> From<TooSmallSparseIndexError<K>> for TryModifyError<K>
where
    K: Key,
{
    fn from(value: TooSmallSparseIndexError<K>) -> Self {
        Self::TooSmallSparseIndex(value)
    }
}

impl<K> From<TryReserveError> for TryModifyError<K>
where
    K: Key,
{
    fn from(value: TryReserveError) -> Self {
        Self::TryReserve(value)
    }
}
