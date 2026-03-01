use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_soa_erased::storage::AllocError;

use crate::archetype::error::{DuplicateComponentError, IncompatibleArchetypeError};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromBundleError<B> {
    pub reason: FromBundleErrorKind,
    pub bundle: B,
}

impl<B> FromBundleError<B> {
    #[inline]
    pub(super) fn new(bundle: B, reason: FromBundleErrorKind) -> Self {
        Self { reason, bundle }
    }
}

impl<B> Display for FromBundleError<B>
where
    B: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, bundle } = self;
        write!(f, "failed to create erased bundle from {bundle}: {reason}")
    }
}

impl<B> Error for FromBundleError<B>
where
    B: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Debug, Clone)]
pub enum FromBundleErrorKind {
    DuplicateComponent(DuplicateComponentError),
    Alloc(AllocError),
}

impl From<DuplicateComponentError> for FromBundleErrorKind {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<AllocError> for FromBundleErrorKind {
    #[inline]
    fn from(error: AllocError) -> Self {
        Self::Alloc(error)
    }
}

impl Display for FromBundleErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::Alloc(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromBundleErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::Alloc(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DowncastError<T>
where
    T: ?Sized,
{
    pub reason: IncompatibleArchetypeError,
    pub value: T,
}

impl<T> DowncastError<T> {
    #[inline]
    pub(super) fn new(value: T, reason: IncompatibleArchetypeError) -> Self {
        Self { reason, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> DowncastError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { reason, value } = self;
        DowncastError::new(f(value), reason)
    }
}

impl<T> From<DowncastError<T>> for IncompatibleArchetypeError {
    #[inline]
    fn from(error: DowncastError<T>) -> Self {
        let DowncastError { reason, .. } = error;
        reason
    }
}

impl<T> Display for DowncastError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to downcast {value} into component: {reason}")
    }
}

impl<T> Error for DowncastError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}
