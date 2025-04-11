use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::error::{LayoutMismatchError, LenMismatchError};

#[derive(Debug, Clone)]
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

impl<T> Display for IntoValueError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to convert {value}: {reason}")
    }
}

impl<T> Error for IntoValueError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
pub enum IntoValueErrorKind {
    LayoutMismatch(LayoutMismatchError),
    LenMismatch(LenMismatchError),
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
            Self::LayoutMismatch(error) => f.debug_tuple("LayoutMismatch").field(error).finish(),
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
        }
    }
}

impl Display for IntoValueErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IntoValueErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutMismatch(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
        }
    }
}
