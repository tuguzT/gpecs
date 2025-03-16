use alloc::collections::TryReserveError as AllocTryReserveError;
use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::{key::Key, soa::vec::TryReserveError as SoaTryReserveError};

#[derive(Clone, PartialEq, Eq, Debug)]
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
            Self::Sparse(e) => write!(f, "sparse: {}", e),
            Self::Dense(e) => write!(f, "dense: {}", e),
        }
    }
}

impl Error for TryReserveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sparse(e) => Some(e),
            Self::Dense(e) => Some(e),
        }
    }
}

pub struct TooLargeSparseIndexError<K>
where
    K: Key,
{
    pub inner: <K::SparseIndex as TryInto<usize>>::Error,
}

impl<K> TooLargeSparseIndexError<K>
where
    K: Key,
{
    #[inline]
    pub fn new(inner: <K::SparseIndex as TryInto<usize>>::Error) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> <K::SparseIndex as TryInto<usize>>::Error {
        let Self { inner } = self;
        inner
    }
}

impl<K> Debug for TooLargeSparseIndexError<K>
where
    K: Key,
    <K::SparseIndex as TryInto<usize>>::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TooLargeSparseIndexError")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<K> Display for TooLargeSparseIndexError<K>
where
    K: Key,
    <K::SparseIndex as TryInto<usize>>::Error: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        write!(f, "sparse index is too large for `usize`: {inner}")
    }
}

impl<K> Error for TooLargeSparseIndexError<K>
where
    K: Key,
    <K::SparseIndex as TryInto<usize>>::Error: Error,
{
}

pub struct TooSmallSparseIndexError<K>
where
    K: Key,
{
    pub inner: <K::SparseIndex as TryFrom<usize>>::Error,
}

impl<K> TooSmallSparseIndexError<K>
where
    K: Key,
{
    #[inline]
    pub fn new(inner: <K::SparseIndex as TryFrom<usize>>::Error) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> <K::SparseIndex as TryFrom<usize>>::Error {
        let Self { inner } = self;
        inner
    }
}

impl<K> Debug for TooSmallSparseIndexError<K>
where
    K: Key,
    <K::SparseIndex as TryFrom<usize>>::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TooSmallSparseIndexError")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<K> Display for TooSmallSparseIndexError<K>
where
    K: Key,
    <K::SparseIndex as TryFrom<usize>>::Error: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        write!(f, "sparse index is too small for `usize`: {inner}")
    }
}

impl<K> Error for TooSmallSparseIndexError<K>
where
    K: Key,
    <K::SparseIndex as TryFrom<usize>>::Error: Error,
{
}

pub enum InvalidKeyError<K>
where
    K: Key,
{
    TooLargeSparseIndex(TooLargeSparseIndexError<K>),
    TooSmallSparseIndex(TooSmallSparseIndexError<K>),
}

impl<K> Debug for InvalidKeyError<K>
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
        }
    }
}

impl<K> Display for InvalidKeyError<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Display,
    TooSmallSparseIndexError<K>: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLargeSparseIndex(error) => Display::fmt(error, f),
            Self::TooSmallSparseIndex(error) => Display::fmt(error, f),
        }
    }
}

impl<K> Error for InvalidKeyError<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Error,
    TooSmallSparseIndexError<K>: Error,
{
}

impl<K> From<TooLargeSparseIndexError<K>> for InvalidKeyError<K>
where
    K: Key,
{
    fn from(value: TooLargeSparseIndexError<K>) -> Self {
        Self::TooLargeSparseIndex(value)
    }
}

impl<K> From<TooSmallSparseIndexError<K>> for InvalidKeyError<K>
where
    K: Key,
{
    fn from(value: TooSmallSparseIndexError<K>) -> Self {
        Self::TooSmallSparseIndex(value)
    }
}

pub enum TryInvalidKeyError<K>
where
    K: Key,
{
    TooLargeSparseIndex(TooLargeSparseIndexError<K>),
    TooSmallSparseIndex(TooSmallSparseIndexError<K>),
    TryReserve(TryReserveError),
}

impl<K> Debug for TryInvalidKeyError<K>
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

impl<K> Display for TryInvalidKeyError<K>
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

impl<K> Error for TryInvalidKeyError<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Error,
    TooSmallSparseIndexError<K>: Error,
{
}

impl<K> From<TooLargeSparseIndexError<K>> for TryInvalidKeyError<K>
where
    K: Key,
{
    fn from(value: TooLargeSparseIndexError<K>) -> Self {
        Self::TooLargeSparseIndex(value)
    }
}

impl<K> From<TooSmallSparseIndexError<K>> for TryInvalidKeyError<K>
where
    K: Key,
{
    fn from(value: TooSmallSparseIndexError<K>) -> Self {
        Self::TooSmallSparseIndex(value)
    }
}

impl<K> From<TryReserveError> for TryInvalidKeyError<K>
where
    K: Key,
{
    fn from(value: TryReserveError) -> Self {
        Self::TryReserve(value)
    }
}

impl<K> From<InvalidKeyError<K>> for TryInvalidKeyError<K>
where
    K: Key,
{
    fn from(value: InvalidKeyError<K>) -> Self {
        match value {
            InvalidKeyError::TooLargeSparseIndex(error) => Self::TooLargeSparseIndex(error),
            InvalidKeyError::TooSmallSparseIndex(error) => Self::TooSmallSparseIndex(error),
        }
    }
}
