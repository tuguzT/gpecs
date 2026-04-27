use core::{
    error::Error,
    fmt::{self, Display},
};

use gpecs_component::erased::error::{
    ComponentMismatchError, DowncastErrorKind as ComponentDowncastErrorKind, NotRegisteredError,
};
use gpecs_soa_erased::error::LayoutMismatchError;

use crate::erased::error::DuplicateComponentError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DowncastError {
    DuplicateComponent(DuplicateComponentError),
    ComponentNotRegistered(NotRegisteredError),
    ComponentMismatch(ComponentMismatchError),
    LayoutMismatch(LayoutMismatchError),
}

impl From<DuplicateComponentError> for DowncastError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<NotRegisteredError> for DowncastError {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl From<ComponentMismatchError> for DowncastError {
    #[inline]
    fn from(error: ComponentMismatchError) -> Self {
        Self::ComponentMismatch(error)
    }
}

impl From<LayoutMismatchError> for DowncastError {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<ComponentDowncastErrorKind> for DowncastError {
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

impl Display for DowncastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
            Self::ComponentMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for DowncastError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
            Self::ComponentMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
        }
    }
}
