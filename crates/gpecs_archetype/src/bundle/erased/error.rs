use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_component::erased::error::{
    ComponentMismatchError, DowncastErrorKind as ComponentDowncastErrorKind, NotRegisteredError,
};
use gpecs_soa_erased::error::LayoutMismatchError;

use crate::{
    bundle::error::DowncastError as BundleDowncastError,
    erased::error::{
        DuplicateComponentError, IncompatibleArchetypeError, IncompatibleArchetypeViewExactError,
        MissingComponentError,
    },
};

#[cfg(feature = "alloc")]
pub use crate::alloc::bundle::erased::{
    from_bundle::{FromBundleError, FromBundleErrorKind},
    from_components::FromComponentsError,
    insert::{InsertError, InsertErrorKind},
    remove::{RemoveError, RemoveErrorKind},
    replace::{ReplaceError, ReplaceErrorKind},
};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DowncastError<T>
where
    T: ?Sized,
{
    pub source: DowncastErrorKind,
    pub value: T,
}

impl<T> DowncastError<T> {
    #[inline]
    pub(super) fn new(value: T, source: DowncastErrorKind) -> Self {
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

    #[inline]
    pub fn into_source(self) -> DowncastErrorKind {
        let Self { source, .. } = self;
        source
    }
}

impl<T> Display for DowncastError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, value } = self;
        write!(f, "failed to downcast {value} into bundle: {source}")
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DowncastErrorKind {
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
    ComponentNotRegistered(NotRegisteredError),
    ComponentMismatch(ComponentMismatchError),
    LayoutMismatch(LayoutMismatchError),
}

impl From<DuplicateComponentError> for DowncastErrorKind {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<MissingComponentError> for DowncastErrorKind {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<NotRegisteredError> for DowncastErrorKind {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl From<ComponentMismatchError> for DowncastErrorKind {
    #[inline]
    fn from(error: ComponentMismatchError) -> Self {
        Self::ComponentMismatch(error)
    }
}

impl From<LayoutMismatchError> for DowncastErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<IncompatibleArchetypeError> for DowncastErrorKind {
    #[inline]
    fn from(error: IncompatibleArchetypeError) -> Self {
        use IncompatibleArchetypeError::{
            ComponentNotRegistered, DuplicateComponent, MissingComponent,
        };

        match error {
            DuplicateComponent(error) => Self::DuplicateComponent(error),
            MissingComponent(error) => Self::MissingComponent(error),
            ComponentNotRegistered(error) => Self::ComponentNotRegistered(error),
        }
    }
}

impl From<ComponentDowncastErrorKind> for DowncastErrorKind {
    #[inline]
    fn from(error: ComponentDowncastErrorKind) -> Self {
        use ComponentDowncastErrorKind::{
            ComponentMismatch, ComponentNotRegistered, LayoutMismatch,
        };

        match error {
            ComponentNotRegistered(error) => Self::ComponentNotRegistered(error),
            ComponentMismatch(error) => Self::ComponentMismatch(error),
            LayoutMismatch(error) => Self::LayoutMismatch(error),
        }
    }
}

impl From<BundleDowncastError> for DowncastErrorKind {
    #[inline]
    fn from(error: BundleDowncastError) -> Self {
        use BundleDowncastError::{
            ComponentMismatch, ComponentNotRegistered, DuplicateComponent, LayoutMismatch,
        };

        match error {
            DuplicateComponent(error) => Self::DuplicateComponent(error),
            ComponentNotRegistered(error) => Self::ComponentNotRegistered(error),
            ComponentMismatch(error) => Self::ComponentMismatch(error),
            LayoutMismatch(error) => Self::LayoutMismatch(error),
        }
    }
}

impl Display for DowncastErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
            Self::ComponentMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for DowncastErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
            Self::ComponentMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
        }
    }
}

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
