use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::erased::error::IncompatibleArchetypeViewExactError;

#[cfg(feature = "alloc")]
pub use crate::alloc::bundle::erased::{
    downcast::{DowncastError, DowncastErrorKind},
    from_bundle::{FromBundleError, FromBundleErrorKind},
    from_components::FromComponentsError,
    insert::{InsertError, InsertErrorKind},
    remove::{RemoveError, RemoveErrorKind},
    replace::{ReplaceError, ReplaceErrorKind},
};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ShuffleError<T, A, E> {
    pub source: ShuffleErrorKind<E>,
    pub bundle: T,
    pub archetype: A,
}

impl<T, A, E> From<ShuffleError<T, A, E>> for ShuffleErrorKind<E> {
    #[inline]
    fn from(error: ShuffleError<T, A, E>) -> Self {
        let ShuffleError { source, .. } = error;
        source
    }
}

impl<T, A, E> Display for ShuffleError<T, A, E>
where
    T: Display,
    A: Display,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            source,
            bundle,
            archetype,
        } = self;

        write!(f, "failed to shuffle {bundle} by {archetype}: {source}")
    }
}

impl<T, A, E> Error for ShuffleError<T, A, E>
where
    T: Debug + Display,
    A: Debug + Display,
    E: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        source.source()
    }

    #[expect(deprecated)]
    fn cause(&self) -> Option<&dyn Error> {
        let Self { source, .. } = self;
        source.cause()
    }
}

#[derive(Debug, Clone)]
pub enum ShuffleErrorKind<T> {
    IncompatibleArchetype(IncompatibleArchetypeViewExactError),
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<IncompatibleArchetypeViewExactError> for ShuffleErrorKind<T> {
    #[inline]
    fn from(error: IncompatibleArchetypeViewExactError) -> Self {
        Self::IncompatibleArchetype(error)
    }
}

impl<T> From<LayoutError> for ShuffleErrorKind<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> Display for ShuffleErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompatibleArchetype(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for ShuffleErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IncompatibleArchetype(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::IncompatibleArchetype(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}
