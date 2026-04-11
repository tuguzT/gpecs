use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_component::erased::error::{
    ComponentMismatchError, DowncastErrorKind as ComponentDowncastErrorKind, NotRegisteredError,
};
use gpecs_soa_erased::error::LayoutMismatchError;

use crate::erased::error::{
    DuplicateComponentError, IncompatibleArchetypeError, MissingComponentError,
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
}

impl<T> From<DowncastError<T>> for DowncastErrorKind {
    #[inline]
    fn from(error: DowncastError<T>) -> Self {
        let DowncastError { source, .. } = error;
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
