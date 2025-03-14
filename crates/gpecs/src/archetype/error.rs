use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::{
    bundle::{error::DuplicateComponentError, Bundle},
    component::registry::ComponentId,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct ExclusiveComponentError {
    pub(super) component_id: ComponentId,
}

impl ExclusiveComponentError {
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

impl Display for ExclusiveComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { component_id } = *self;
        write!(f, "component {component_id:?} is exclusive to this bundle")
    }
}

impl Error for ExclusiveComponentError {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[non_exhaustive]
pub enum IncompatibleBundleError {
    DuplicateComponent(DuplicateComponentError),
    ExclusiveComponent(ExclusiveComponentError),
}

impl From<DuplicateComponentError> for IncompatibleBundleError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<ExclusiveComponentError> for IncompatibleBundleError {
    #[inline]
    fn from(error: ExclusiveComponentError) -> Self {
        Self::ExclusiveComponent(error)
    }
}

impl Display for IncompatibleBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incompatible bundle: ")?;
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::ExclusiveComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IncompatibleBundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::ExclusiveComponent(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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
        write!(f, "too few components in this bundle")
    }
}

impl Error for TooFewComponentsError {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[non_exhaustive]
pub enum IncompatibleBundleExactError {
    DuplicateComponent(DuplicateComponentError),
    ExclusiveComponent(ExclusiveComponentError),
    TooFewComponents(TooFewComponentsError),
}

impl From<DuplicateComponentError> for IncompatibleBundleExactError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<ExclusiveComponentError> for IncompatibleBundleExactError {
    #[inline]
    fn from(error: ExclusiveComponentError) -> Self {
        Self::ExclusiveComponent(error)
    }
}

impl From<TooFewComponentsError> for IncompatibleBundleExactError {
    #[inline]
    fn from(error: TooFewComponentsError) -> Self {
        Self::TooFewComponents(error)
    }
}

impl From<IncompatibleBundleError> for IncompatibleBundleExactError {
    #[inline]
    fn from(error: IncompatibleBundleError) -> Self {
        match error {
            IncompatibleBundleError::DuplicateComponent(error) => Self::DuplicateComponent(error),
            IncompatibleBundleError::ExclusiveComponent(error) => Self::ExclusiveComponent(error),
        }
    }
}

impl Display for IncompatibleBundleExactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incompatible bundle: ")?;
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::ExclusiveComponent(error) => Display::fmt(error, f),
            Self::TooFewComponents(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IncompatibleBundleExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::ExclusiveComponent(error) => Some(error),
            Self::TooFewComponents(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[non_exhaustive]
pub struct IncompatibleBundleValueError<B>
where
    B: Bundle,
{
    pub value: B,
    pub reason: IncompatibleBundleExactError,
}

impl<B> Display for IncompatibleBundleValueError<B>
where
    B: Bundle + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, reason } = self;

        let Some(reason) = reason.source() else {
            unreachable!("incompatible bundle exact error should have a source")
        };
        write!(f, "incompatible bundle value is {value}, reason: {reason}")
    }
}

impl<B> Error for IncompatibleBundleValueError<B>
where
    B: Bundle + Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        reason.source()
    }
}
