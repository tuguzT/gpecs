use core::{
    alloc::{Layout, LayoutError},
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::error::{
    InsufficientAlignError, LayoutMismatchError, LenMismatchError, NotAlignedError, check_layout,
};

#[derive(Debug, Clone)]
pub enum DataError {
    InvalidLayout(LayoutError),
    NotAligned(NotAlignedError),
    LenMismatch(LenMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<LayoutError> for DataError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<NotAlignedError> for DataError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LenMismatchError> for DataError {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<InsufficientAlignError> for DataError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for DataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::NotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DowncastError<T>
where
    T: ?Sized,
{
    pub source: LayoutMismatchError,
    pub value: T,
}

impl<T> DowncastError<T> {
    #[inline]
    fn new(value: T, source: LayoutMismatchError) -> Self {
        Self { source, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> DowncastError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { source, value } = self;
        DowncastError::new(f(value), source)
    }
}

impl<T> Display for DowncastError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, value } = self;
        write!(f, "failed to downcast {value}: {source}")
    }
}

impl<T> Error for DowncastError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[inline]
pub(super) fn check_downcast<T, U>(layout: Layout, value: U) -> Result<U, DowncastError<U>> {
    let expected = Layout::new::<T>();
    match check_layout(layout, expected) {
        Ok(()) => Ok(value),
        Err(source) => Err(DowncastError::new(value, source)),
    }
}

#[derive(Debug, Clone)]
pub enum TryFromPtrError {
    NotAligned(NotAlignedError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for TryFromPtrError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<InsufficientAlignError> for TryFromPtrError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for TryFromPtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for TryFromPtrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TryFromSlicePtrError {
    InvalidLayout(LayoutError),
    NotAligned(NotAlignedError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<LayoutError> for TryFromSlicePtrError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<NotAlignedError> for TryFromSlicePtrError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<InsufficientAlignError> for TryFromSlicePtrError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for TryFromSlicePtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for TryFromSlicePtrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::NotAligned(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FromLayoutDataError<T> {
    LenMismatch(LenMismatchError),
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T),
}

impl<T> From<LenMismatchError> for FromLayoutDataError<T> {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl<T> From<InsufficientAlignError> for FromLayoutDataError<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> Display for FromLayoutDataError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromLayoutDataError<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromValueError<T, V>
where
    V: ?Sized,
{
    pub source: FromValueErrorKind<T>,
    pub value: V,
}

impl<T, V> FromValueError<T, V> {
    #[inline]
    pub(crate) fn new(source: FromValueErrorKind<T>, value: V) -> Self {
        Self { source, value }
    }
}

impl<T, V> Display for FromValueError<T, V>
where
    T: Display,
    V: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, value } = self;
        write!(f, "failed to convert {value}: {source}")
    }
}

impl<T, V> Error for FromValueError<T, V>
where
    T: Error,
    V: Debug + Display + ?Sized,
{
    fn cause(&self) -> Option<&dyn Error> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum FromValueErrorKind<T> {
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T),
}

impl<T> From<InsufficientAlignError> for FromValueErrorKind<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> Display for FromValueErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromValueErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromStorageError<T>
where
    T: ?Sized,
{
    pub source: FromStorageErrorKind,
    pub storage: T,
}

impl<T> FromStorageError<T> {
    pub(crate) fn new(source: FromStorageErrorKind, storage: T) -> Self {
        Self { source, storage }
    }
}

impl<T> Display for FromStorageError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, storage } = self;
        write!(f, "failed to create erased field with {storage}: {source}")
    }
}

impl<T> Error for FromStorageError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum FromStorageErrorKind {
    NotAligned(NotAlignedError),
    LenMismatch(LenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for FromStorageErrorKind {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LenMismatchError> for FromStorageErrorKind {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for FromStorageErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<InsufficientAlignError> for FromStorageErrorKind {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for FromStorageErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromStorageErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}
