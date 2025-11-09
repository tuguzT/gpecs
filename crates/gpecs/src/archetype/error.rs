use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::{bundle::Bundle, component::registry::ComponentId};

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
        write!(f, "entity already has {component_id}")
    }
}

impl Error for AlreadyHasComponentError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InsertBundleExactErrorKind {
    DuplicateComponent(DuplicateComponentError),
    AlreadyHasComponent(AlreadyHasComponentError),
}

impl From<DuplicateComponentError> for InsertBundleExactErrorKind {
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<AlreadyHasComponentError> for InsertBundleExactErrorKind {
    fn from(error: AlreadyHasComponentError) -> Self {
        Self::AlreadyHasComponent(error)
    }
}

impl Display for InsertBundleExactErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::AlreadyHasComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for InsertBundleExactErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::AlreadyHasComponent(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct InsertBundleExactError<B>
where
    B: Bundle,
{
    pub value: B,
    pub kind: InsertBundleExactErrorKind,
}

impl<B> Display for InsertBundleExactError<B>
where
    B: Bundle + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, kind } = self;
        write!(f, "exact bundle {value} cannot be inserted: {kind}")
    }
}

impl<B> Error for InsertBundleExactError<B>
where
    B: Bundle + Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { kind, .. } = self;
        kind.source()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct InsertBundleError<B>
where
    B: Bundle,
{
    pub value: B,
    pub reason: DuplicateComponentError,
}

impl<B> Display for InsertBundleError<B>
where
    B: Bundle + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, reason } = self;
        write!(f, "bundle {value} cannot be inserted: {reason}")
    }
}

impl<B> Error for InsertBundleError<B>
where
    B: Bundle + Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        reason.source()
    }
}

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
        write!(f, "{component_id} is exclusive to this bundle")
    }
}

impl Error for MissingComponentError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RemoveBundleExactError {
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
}

impl From<DuplicateComponentError> for RemoveBundleExactError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<MissingComponentError> for RemoveBundleExactError {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl Display for RemoveBundleExactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exact bundle cannot be removed: ")?;
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for RemoveBundleExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct ComponentNotRegisteredError;

impl Display for ComponentNotRegisteredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "component was not registered")
    }
}

impl Error for ComponentNotRegisteredError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GetComponentsError {
    DuplicateComponent(DuplicateComponentError),
    ComponentNotRegistered(ComponentNotRegisteredError),
}

impl From<DuplicateComponentError> for GetComponentsError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<ComponentNotRegisteredError> for GetComponentsError {
    #[inline]
    fn from(error: ComponentNotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl Display for GetComponentsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
        }
    }
}

impl Error for GetComponentsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IncompatibleBundleError {
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
    ComponentNotRegistered(ComponentNotRegisteredError),
}

impl From<DuplicateComponentError> for IncompatibleBundleError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<MissingComponentError> for IncompatibleBundleError {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<GetComponentsError> for IncompatibleBundleError {
    #[inline]
    fn from(error: GetComponentsError) -> Self {
        match error {
            GetComponentsError::DuplicateComponent(error) => Self::DuplicateComponent(error),
            GetComponentsError::ComponentNotRegistered(error) => {
                Self::ComponentNotRegistered(error)
            }
        }
    }
}

impl From<ComponentNotRegisteredError> for IncompatibleBundleError {
    #[inline]
    fn from(error: ComponentNotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl Display for IncompatibleBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incompatible bundle: ")?;
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IncompatibleBundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
        }
    }
}

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
        write!(f, "too few components in this bundle")
    }
}

impl Error for TooFewComponentsError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IncompatibleBundleExactError {
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
    ComponentNotRegistered(ComponentNotRegisteredError),
    TooFewComponents(TooFewComponentsError),
}

impl From<DuplicateComponentError> for IncompatibleBundleExactError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<MissingComponentError> for IncompatibleBundleExactError {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<ComponentNotRegisteredError> for IncompatibleBundleExactError {
    #[inline]
    fn from(error: ComponentNotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl From<GetComponentsError> for IncompatibleBundleExactError {
    #[inline]
    fn from(error: GetComponentsError) -> Self {
        match error {
            GetComponentsError::DuplicateComponent(error) => Self::DuplicateComponent(error),
            GetComponentsError::ComponentNotRegistered(error) => {
                Self::ComponentNotRegistered(error)
            }
        }
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
            IncompatibleBundleError::MissingComponent(error) => Self::MissingComponent(error),
            IncompatibleBundleError::ComponentNotRegistered(error) => {
                Self::ComponentNotRegistered(error)
            }
        }
    }
}

impl Display for IncompatibleBundleExactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incompatible bundle: ")?;
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
            Self::TooFewComponents(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IncompatibleBundleExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
            Self::TooFewComponents(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
        write!(f, "incompatible bundle {value}: {reason}")
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
