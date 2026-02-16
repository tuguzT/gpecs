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

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub enum TryModifyErrorKind<K>
where
    K: Key,
{
    TooLargeSparseIndex(TooLargeSparseIndexError<K>),
    TooSmallSparseIndex(TooSmallSparseIndexError<K>),
    TryReserve(TryReserveError),
}

impl<K> Debug for TryModifyErrorKind<K>
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

impl<K> Display for TryModifyErrorKind<K>
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

impl<K> Error for TryModifyErrorKind<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Error,
    TooSmallSparseIndexError<K>: Error,
{
}

impl<K> From<TooLargeSparseIndexError<K>> for TryModifyErrorKind<K>
where
    K: Key,
{
    fn from(value: TooLargeSparseIndexError<K>) -> Self {
        Self::TooLargeSparseIndex(value)
    }
}

impl<K> From<TooSmallSparseIndexError<K>> for TryModifyErrorKind<K>
where
    K: Key,
{
    fn from(value: TooSmallSparseIndexError<K>) -> Self {
        Self::TooSmallSparseIndex(value)
    }
}

impl<K> From<TryReserveError> for TryModifyErrorKind<K>
where
    K: Key,
{
    fn from(value: TryReserveError) -> Self {
        Self::TryReserve(value)
    }
}

#[non_exhaustive]
pub struct TryModifyError<K, V>
where
    K: Key,
    V: ?Sized,
{
    pub kind: TryModifyErrorKind<K>,
    pub value: V,
}

impl<K, V> TryModifyError<K, V>
where
    K: Key,
{
    #[inline]
    pub(super) fn new(kind: TryModifyErrorKind<K>, value: V) -> Self {
        Self { kind, value }
    }

    #[inline]
    pub fn map_value<F, U>(self, f: F) -> TryModifyError<K, U>
    where
        F: FnOnce(V) -> U,
    {
        let Self { kind, value } = self;
        let value = f(value);
        TryModifyError { kind, value }
    }
}

impl<K, V> Debug for TryModifyError<K, V>
where
    K: Key,
    V: Debug + ?Sized,
    TryModifyErrorKind<K>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { kind, value } = self;
        f.debug_struct("TryModifyError")
            .field("kind", kind)
            .field("value", &value)
            .finish()
    }
}

impl<K, V> Display for TryModifyError<K, V>
where
    K: Key,
    V: Display + ?Sized,
    TryModifyErrorKind<K>: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { kind, value } = self;
        write!(f, "invalid key for value {value} ({kind})")
    }
}

impl<K, V> Error for TryModifyError<K, V>
where
    K: Key,
    V: Debug + Display + ?Sized,
    TryModifyErrorKind<K>: Error,
{
}
