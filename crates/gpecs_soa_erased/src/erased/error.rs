use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::error::{InvalidLayoutError, LayoutMismatchError, LenMismatchError};

#[derive(Clone)]
#[non_exhaustive]
pub struct FromValueError<T>
where
    T: ?Sized,
{
    pub reason: InvalidLayoutError,
    pub value: T,
}

impl<T> FromValueError<T> {
    #[inline]
    pub(crate) fn new(value: T, reason: InvalidLayoutError) -> Self {
        Self { reason, value }
    }
}

impl<T> Debug for FromValueError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, .. } = self;
        Debug::fmt(reason, f)
    }
}

impl<T> Display for FromValueError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, .. } = self;
        Display::fmt(reason, f)
    }
}

impl<T> Error for FromValueError<T>
where
    T: ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct IntoValueError<T>
where
    T: ?Sized,
{
    pub reason: IntoValueErrorKind,
    pub value: T,
}

impl<T> IntoValueError<T> {
    #[inline]
    pub(crate) fn new(value: T, reason: IntoValueErrorKind) -> Self {
        Self { reason, value }
    }
}

impl<T> Debug for IntoValueError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, .. } = self;
        Debug::fmt(reason, f)
    }
}

impl<T> Display for IntoValueError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, .. } = self;
        Display::fmt(reason, f)
    }
}

impl<T> Error for IntoValueError<T>
where
    T: ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
pub enum IntoValueErrorKind {
    InvalidLayout(InvalidLayoutError),
    LayoutMismatch(LayoutMismatchError),
    LenMismatch(LenMismatchError),
}

impl From<InvalidLayoutError> for IntoValueErrorKind {
    fn from(error: InvalidLayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<LayoutMismatchError> for IntoValueErrorKind {
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<LenMismatchError> for IntoValueErrorKind {
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl Debug for IntoValueErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::InvalidLayout(error) => f.debug_tuple("InvalidLayout").field(error).finish(),
            Self::LayoutMismatch(error) => f.debug_tuple("LayoutMismatch").field(error).finish(),
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
        }
    }
}

impl Display for IntoValueErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IntoValueErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
        }
    }
}

#[derive(Clone)]
pub enum ErasedSoaError {
    LenMismatch(LenMismatchError),
    InvalidLayout(InvalidLayoutError),
}

impl From<LenMismatchError> for ErasedSoaError {
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<InvalidLayoutError> for ErasedSoaError {
    fn from(error: InvalidLayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Debug for ErasedSoaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
            Self::InvalidLayout(error) => f.debug_tuple("InvalidLayout").field(error).finish(),
        }
    }
}

impl Display for ErasedSoaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedSoaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}
