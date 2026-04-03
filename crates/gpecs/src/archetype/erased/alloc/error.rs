use std::{
    error::Error,
    fmt::{self, Display},
};

use crate::{
    archetype::erased::error::{
        IncompatibleArchetypeViewExactError, MissingComponentError, TooFewComponentsError,
    },
    component::{erased::error::NotRegisteredError, registry::ComponentId},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DuplicateComponentError {
    component_id: ComponentId,
}

impl DuplicateComponentError {
    #[inline]
    pub fn new(component_id: ComponentId) -> Self {
        Self { component_id }
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id } = *self;
        component_id
    }
}

impl Display for DuplicateComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { component_id } = *self;
        write!(f, "duplicate {component_id} were found")
    }
}

impl Error for DuplicateComponentError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArchetypeError {
    DuplicateComponent(DuplicateComponentError),
    ComponentNotRegistered(NotRegisteredError),
}

impl From<DuplicateComponentError> for ArchetypeError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<NotRegisteredError> for ArchetypeError {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl Display for ArchetypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ArchetypeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IncompatibleArchetypeError {
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
    ComponentNotRegistered(NotRegisteredError),
}

impl From<DuplicateComponentError> for IncompatibleArchetypeError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<MissingComponentError> for IncompatibleArchetypeError {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<NotRegisteredError> for IncompatibleArchetypeError {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl From<ArchetypeError> for IncompatibleArchetypeError {
    #[inline]
    fn from(error: ArchetypeError) -> Self {
        match error {
            ArchetypeError::DuplicateComponent(error) => Self::DuplicateComponent(error),
            ArchetypeError::ComponentNotRegistered(error) => Self::ComponentNotRegistered(error),
        }
    }
}

impl Display for IncompatibleArchetypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incompatible archetype: ")?;
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IncompatibleArchetypeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IncompatibleArchetypeExactError {
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
    ComponentNotRegistered(NotRegisteredError),
    TooFewComponents(TooFewComponentsError),
}

impl From<DuplicateComponentError> for IncompatibleArchetypeExactError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<MissingComponentError> for IncompatibleArchetypeExactError {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<NotRegisteredError> for IncompatibleArchetypeExactError {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl From<TooFewComponentsError> for IncompatibleArchetypeExactError {
    #[inline]
    fn from(error: TooFewComponentsError) -> Self {
        Self::TooFewComponents(error)
    }
}

impl From<ArchetypeError> for IncompatibleArchetypeExactError {
    #[inline]
    fn from(error: ArchetypeError) -> Self {
        match error {
            ArchetypeError::DuplicateComponent(error) => Self::DuplicateComponent(error),
            ArchetypeError::ComponentNotRegistered(error) => Self::ComponentNotRegistered(error),
        }
    }
}

impl From<IncompatibleArchetypeViewExactError> for IncompatibleArchetypeExactError {
    #[inline]
    fn from(error: IncompatibleArchetypeViewExactError) -> Self {
        use IncompatibleArchetypeViewExactError::{MissingComponent, TooFewComponents};

        match error {
            MissingComponent(error) => Self::MissingComponent(error),
            TooFewComponents(error) => Self::TooFewComponents(error),
        }
    }
}

impl From<IncompatibleArchetypeError> for IncompatibleArchetypeExactError {
    #[inline]
    fn from(error: IncompatibleArchetypeError) -> Self {
        match error {
            IncompatibleArchetypeError::MissingComponent(error) => Self::MissingComponent(error),
            IncompatibleArchetypeError::DuplicateComponent(error) => {
                Self::DuplicateComponent(error)
            }
            IncompatibleArchetypeError::ComponentNotRegistered(error) => {
                Self::ComponentNotRegistered(error)
            }
        }
    }
}

impl Display for IncompatibleArchetypeExactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incompatible exact archetype: ")?;
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
            Self::TooFewComponents(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IncompatibleArchetypeExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
            Self::TooFewComponents(error) => Some(error),
        }
    }
}
