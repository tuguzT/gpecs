use core::{
    error::Error,
    fmt::{self, Display},
};

use gpecs_component::registry::ComponentId;

#[cfg(feature = "alloc")]
pub use crate::erased::alloc::error::{
    ArchetypeError, DuplicateComponentError, IncompatibleArchetypeError,
    IncompatibleArchetypeExactError,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MissingComponentError {
    component_id: ComponentId,
}

impl MissingComponentError {
    #[inline]
    pub fn new(component_id: ComponentId) -> Self {
        Self { component_id }
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }
}

impl Display for MissingComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { component_id } = *self;
        write!(f, "{component_id} is missing")
    }
}

impl Error for MissingComponentError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AlreadyHasComponentError {
    component_id: ComponentId,
}

impl AlreadyHasComponentError {
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

impl Display for AlreadyHasComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { component_id } = *self;
        write!(f, "already has {component_id}")
    }
}

impl Error for AlreadyHasComponentError {}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[non_exhaustive]
pub struct TooFewComponentsError;

impl TooFewComponentsError {
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

impl Display for TooFewComponentsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "too few components in archetype")
    }
}

impl Error for TooFewComponentsError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IncompatibleArchetypeViewExactError {
    MissingComponent(MissingComponentError),
    TooFewComponents(TooFewComponentsError),
}

impl From<MissingComponentError> for IncompatibleArchetypeViewExactError {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<TooFewComponentsError> for IncompatibleArchetypeViewExactError {
    #[inline]
    fn from(error: TooFewComponentsError) -> Self {
        Self::TooFewComponents(error)
    }
}

impl Display for IncompatibleArchetypeViewExactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incompatible exact archetype view: ")?;
        match self {
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::TooFewComponents(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IncompatibleArchetypeViewExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingComponent(error) => Some(error),
            Self::TooFewComponents(error) => Some(error),
        }
    }
}
