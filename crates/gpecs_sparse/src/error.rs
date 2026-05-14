use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

#[cfg(feature = "alloc")]
pub use crate::alloc::error::{TryModifyError, TryModifyErrorKind, TryReserveError};

use crate::key::Key;

#[non_exhaustive]
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
        let Self { inner } = self;
        f.debug_struct("TooLargeSparseIndexError")
            .field("inner", inner)
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

#[non_exhaustive]
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
        let Self { inner } = self;
        f.debug_struct("TooSmallSparseIndexError")
            .field("inner", inner)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseIndexOutOfBoundsError {
    dense_index: usize,
    dense_len: usize,
}

impl DenseIndexOutOfBoundsError {
    #[inline]
    pub(crate) fn new(dense_index: usize, dense_len: usize) -> Self {
        Self {
            dense_index,
            dense_len,
        }
    }

    #[inline]
    pub fn dense_index(&self) -> usize {
        let Self { dense_index, .. } = *self;
        dense_index
    }

    #[inline]
    pub fn dense_len(&self) -> usize {
        let Self { dense_len, .. } = *self;
        dense_len
    }
}

impl Display for DenseIndexOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            dense_index,
            dense_len,
        } = *self;
        write!(
            f,
            "dense index {dense_index} is out of bounds for dense slice of length {dense_len}",
        )
    }
}

impl Error for DenseIndexOutOfBoundsError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseIndexOutOfBoundsError {
    sparse_index: usize,
    sparse_len: usize,
}

impl SparseIndexOutOfBoundsError {
    #[inline]
    pub(crate) fn new(sparse_index: usize, sparse_len: usize) -> Self {
        Self {
            sparse_index,
            sparse_len,
        }
    }

    #[inline]
    pub fn sparse_index(&self) -> usize {
        let Self { sparse_index, .. } = *self;
        sparse_index
    }

    #[inline]
    pub fn sparse_len(&self) -> usize {
        let Self { sparse_len, .. } = *self;
        sparse_len
    }
}

impl Display for SparseIndexOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            sparse_index,
            sparse_len,
        } = *self;
        write!(
            f,
            "sparse index {sparse_index} is out of bounds for sparse slice of length {sparse_len}",
        )
    }
}

impl Error for SparseIndexOutOfBoundsError {}

#[derive(Clone, PartialEq, Eq)]
pub struct DenseIndexMismatchError<K>
where
    K: Key,
{
    actual: K::SparseIndex,
    expected: K::SparseIndex,
}

impl<K> DenseIndexMismatchError<K>
where
    K: Key,
{
    #[inline]
    pub(crate) fn new(actual: K::SparseIndex, expected: K::SparseIndex) -> Self {
        Self { actual, expected }
    }

    #[inline]
    pub fn actual(&self) -> K::SparseIndex {
        let Self { actual, .. } = *self;
        actual
    }

    #[inline]
    pub fn expected(&self) -> K::SparseIndex {
        let Self { expected, .. } = *self;
        expected
    }
}

impl<K> Debug for DenseIndexMismatchError<K>
where
    K: Key,
    K::SparseIndex: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, expected } = self;
        f.debug_struct("DenseIndexMismatchError")
            .field("actual", actual)
            .field("expected", expected)
            .finish()
    }
}

impl<K> Display for DenseIndexMismatchError<K>
where
    K: Key,
    K::SparseIndex: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, expected } = *self;
        write!(
            f,
            "dense index {actual} does not match expected dense index {expected}",
        )
    }
}

impl<K> Error for DenseIndexMismatchError<K>
where
    K: Key,
    K::SparseIndex: Debug + Display,
{
}

#[derive(Clone, PartialEq, Eq)]
pub struct SparseIndexMismatchError<K>
where
    K: Key,
{
    actual: K::SparseIndex,
    expected: K::SparseIndex,
}

impl<K> SparseIndexMismatchError<K>
where
    K: Key,
{
    #[inline]
    pub(crate) fn new(actual: K::SparseIndex, expected: K::SparseIndex) -> Self {
        Self { actual, expected }
    }

    #[inline]
    pub fn actual(&self) -> K::SparseIndex {
        let Self { actual, .. } = *self;
        actual
    }

    #[inline]
    pub fn expected(&self) -> K::SparseIndex {
        let Self { expected, .. } = *self;
        expected
    }
}

impl<K> Debug for SparseIndexMismatchError<K>
where
    K: Key,
    K::SparseIndex: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, expected } = self;
        f.debug_struct("SparseIndexMismatchError")
            .field("actual", actual)
            .field("expected", expected)
            .finish()
    }
}

impl<K> Display for SparseIndexMismatchError<K>
where
    K: Key,
    K::SparseIndex: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, expected } = *self;
        write!(
            f,
            "sparse index {actual} does not match expected sparse index {expected}",
        )
    }
}

impl<K> Error for SparseIndexMismatchError<K>
where
    K: Key,
    K::SparseIndex: Debug + Display,
{
}

#[derive(Clone, PartialEq, Eq)]
pub struct EpochMismatchError<K>
where
    K: Key,
{
    actual: K::Epoch,
    expected: K::Epoch,
}

impl<K> EpochMismatchError<K>
where
    K: Key,
{
    #[inline]
    pub(crate) fn new(actual: K::Epoch, expected: K::Epoch) -> Self {
        Self { actual, expected }
    }

    #[inline]
    pub fn actual(&self) -> K::Epoch {
        let Self { actual, .. } = *self;
        actual
    }

    #[inline]
    pub fn expected(&self) -> K::Epoch {
        let Self { expected, .. } = *self;
        expected
    }
}

impl<K> Debug for EpochMismatchError<K>
where
    K: Key,
    K::Epoch: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, expected } = self;
        f.debug_struct("EpochMismatchError")
            .field("actual", actual)
            .field("expected", expected)
            .finish()
    }
}

impl<K> Display for EpochMismatchError<K>
where
    K: Key,
    K::Epoch: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, expected } = *self;
        write!(f, "epoch {actual} does not match expected epoch {expected}")
    }
}

impl<K> Error for EpochMismatchError<K>
where
    K: Key,
    K::Epoch: Debug + Display,
{
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccupiedSparseItemExpectedError {
    sparse_index: usize,
}

impl OccupiedSparseItemExpectedError {
    #[inline]
    pub(crate) fn new(sparse_index: usize) -> Self {
        Self { sparse_index }
    }
}

impl Display for OccupiedSparseItemExpectedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { sparse_index } = *self;
        write!(f, "occupied sparse item expected at {sparse_index}")
    }
}

impl Error for OccupiedSparseItemExpectedError {}

pub enum FromPartsError<K>
where
    K: Key,
{
    TooLargeSparseIndex(TooLargeSparseIndexError<K>),
    TooSmallSparseIndex(TooSmallSparseIndexError<K>),
    OccupiedSparseItemExpected(OccupiedSparseItemExpectedError),
    DenseIndexOutOfBounds(DenseIndexOutOfBoundsError),
    SparseIndexOutOfBounds(SparseIndexOutOfBoundsError),
    DenseIndexMismatch(DenseIndexMismatchError<K>),
    SparseIndexMismatch(SparseIndexMismatchError<K>),
    EpochMismatch(EpochMismatchError<K>),
}

impl<K> Debug for FromPartsError<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Debug,
    TooSmallSparseIndexError<K>: Debug,
    DenseIndexMismatchError<K>: Debug,
    SparseIndexMismatchError<K>: Debug,
    EpochMismatchError<K>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLargeSparseIndex(error) => {
                f.debug_tuple("TooLargeSparseIndex").field(error).finish()
            }
            Self::TooSmallSparseIndex(error) => {
                f.debug_tuple("TooSmallSparseIndex").field(error).finish()
            }
            Self::OccupiedSparseItemExpected(error) => f
                .debug_tuple("OccupiedSparseItemExpected")
                .field(error)
                .finish(),
            Self::DenseIndexOutOfBounds(error) => {
                f.debug_tuple("DenseIndexOutOfBounds").field(error).finish()
            }
            Self::SparseIndexOutOfBounds(error) => f
                .debug_tuple("SparseIndexOutOfBounds")
                .field(error)
                .finish(),
            Self::DenseIndexMismatch(error) => {
                f.debug_tuple("DenseIndexMismatch").field(error).finish()
            }
            Self::SparseIndexMismatch(error) => {
                f.debug_tuple("SparseIndexMismatch").field(error).finish()
            }
            Self::EpochMismatch(error) => f.debug_tuple("EpochMismatch").field(error).finish(),
        }
    }
}

impl<K> Display for FromPartsError<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Display,
    TooSmallSparseIndexError<K>: Display,
    DenseIndexMismatchError<K>: Display,
    SparseIndexMismatchError<K>: Display,
    EpochMismatchError<K>: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLargeSparseIndex(error) => Display::fmt(error, f),
            Self::TooSmallSparseIndex(error) => Display::fmt(error, f),
            Self::OccupiedSparseItemExpected(error) => Display::fmt(error, f),
            Self::DenseIndexOutOfBounds(error) => Display::fmt(error, f),
            Self::SparseIndexOutOfBounds(error) => Display::fmt(error, f),
            Self::DenseIndexMismatch(error) => Display::fmt(error, f),
            Self::SparseIndexMismatch(error) => Display::fmt(error, f),
            Self::EpochMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl<K> Error for FromPartsError<K>
where
    K: Key,
    TooLargeSparseIndexError<K>: Error,
    TooSmallSparseIndexError<K>: Error,
    DenseIndexMismatchError<K>: Error,
    SparseIndexMismatchError<K>: Error,
    EpochMismatchError<K>: Error,
{
}

impl<K> From<TooLargeSparseIndexError<K>> for FromPartsError<K>
where
    K: Key,
{
    fn from(value: TooLargeSparseIndexError<K>) -> Self {
        Self::TooLargeSparseIndex(value)
    }
}

impl<K> From<TooSmallSparseIndexError<K>> for FromPartsError<K>
where
    K: Key,
{
    fn from(value: TooSmallSparseIndexError<K>) -> Self {
        Self::TooSmallSparseIndex(value)
    }
}

impl<K> From<OccupiedSparseItemExpectedError> for FromPartsError<K>
where
    K: Key,
{
    fn from(value: OccupiedSparseItemExpectedError) -> Self {
        Self::OccupiedSparseItemExpected(value)
    }
}

impl<K> From<DenseIndexOutOfBoundsError> for FromPartsError<K>
where
    K: Key,
{
    fn from(value: DenseIndexOutOfBoundsError) -> Self {
        Self::DenseIndexOutOfBounds(value)
    }
}

impl<K> From<SparseIndexOutOfBoundsError> for FromPartsError<K>
where
    K: Key,
{
    fn from(value: SparseIndexOutOfBoundsError) -> Self {
        Self::SparseIndexOutOfBounds(value)
    }
}

impl<K> From<DenseIndexMismatchError<K>> for FromPartsError<K>
where
    K: Key,
{
    fn from(value: DenseIndexMismatchError<K>) -> Self {
        Self::DenseIndexMismatch(value)
    }
}

impl<K> From<SparseIndexMismatchError<K>> for FromPartsError<K>
where
    K: Key,
{
    fn from(value: SparseIndexMismatchError<K>) -> Self {
        Self::SparseIndexMismatch(value)
    }
}

impl<K> From<EpochMismatchError<K>> for FromPartsError<K>
where
    K: Key,
{
    fn from(value: EpochMismatchError<K>) -> Self {
        Self::EpochMismatch(value)
    }
}
