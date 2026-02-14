use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::{
    archetype::error::{
        AlreadyHasComponentError, DuplicateComponentError,
        IncompatibleBundleError as ArchetypeIncompatibleBundleError,
        InsertBundleError as ArchetypeInsertBundleError,
        InsertBundleExactError as ArchetypeInsertBundleExactError,
        InsertBundleExactErrorKind as ArchetypeInsertBundleExactErrorKind, MissingComponentError,
        RemoveBundleExactError as ArchetypeRemoveBundleExactError,
    },
    bundle::Bundle,
    component::error::ComponentNotRegisteredError,
    entity::Entity,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EntityNotFoundError {
    entity: Entity,
}

impl EntityNotFoundError {
    #[inline]
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    #[inline]
    pub fn entity(&self) -> Entity {
        let Self { entity } = *self;
        entity
    }
}

impl Display for EntityNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { entity } = self;
        write!(f, "{entity} not found")
    }
}

impl Error for EntityNotFoundError {}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum IncompatibleBundleError {
    EntityNotFound(EntityNotFoundError),
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
    ComponentNotRegistered(ComponentNotRegisteredError),
}

impl From<EntityNotFoundError> for IncompatibleBundleError {
    #[inline]
    fn from(error: EntityNotFoundError) -> Self {
        Self::EntityNotFound(error)
    }
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

impl From<ComponentNotRegisteredError> for IncompatibleBundleError {
    #[inline]
    fn from(error: ComponentNotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl From<ArchetypeIncompatibleBundleError> for IncompatibleBundleError {
    #[inline]
    fn from(error: ArchetypeIncompatibleBundleError) -> Self {
        match error {
            ArchetypeIncompatibleBundleError::DuplicateComponent(error) => {
                Self::DuplicateComponent(error)
            }
            ArchetypeIncompatibleBundleError::MissingComponent(error) => {
                Self::MissingComponent(error)
            }
            ArchetypeIncompatibleBundleError::ComponentNotRegistered(error) => {
                Self::ComponentNotRegistered(error)
            }
        }
    }
}

impl Display for IncompatibleBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incompatible bundle: ")?;
        match self {
            Self::EntityNotFound(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IncompatibleBundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EntityNotFound(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InsertBundleExactErrorKind {
    EntityNotFound(EntityNotFoundError),
    DuplicateComponent(DuplicateComponentError),
    AlreadyHasComponent(AlreadyHasComponentError),
}

impl From<EntityNotFoundError> for InsertBundleExactErrorKind {
    #[inline]
    fn from(error: EntityNotFoundError) -> Self {
        Self::EntityNotFound(error)
    }
}

impl From<DuplicateComponentError> for InsertBundleExactErrorKind {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<AlreadyHasComponentError> for InsertBundleExactErrorKind {
    #[inline]
    fn from(error: AlreadyHasComponentError) -> Self {
        Self::AlreadyHasComponent(error)
    }
}

impl From<ArchetypeInsertBundleExactErrorKind> for InsertBundleExactErrorKind {
    #[inline]
    fn from(error: ArchetypeInsertBundleExactErrorKind) -> Self {
        match error {
            ArchetypeInsertBundleExactErrorKind::DuplicateComponent(error) => {
                Self::DuplicateComponent(error)
            }
            ArchetypeInsertBundleExactErrorKind::AlreadyHasComponent(error) => {
                Self::AlreadyHasComponent(error)
            }
        }
    }
}

impl Display for InsertBundleExactErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityNotFound(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::AlreadyHasComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for InsertBundleExactErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EntityNotFound(error) => Some(error),
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

impl<B> From<ArchetypeInsertBundleExactError<B>> for InsertBundleExactError<B>
where
    B: Bundle,
{
    #[inline]
    fn from(error: ArchetypeInsertBundleExactError<B>) -> Self {
        let ArchetypeInsertBundleExactError { value, kind } = error;
        let kind = kind.into();
        Self { value, kind }
    }
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
pub enum InsertBundleErrorKind {
    EntityNotFound(EntityNotFoundError),
    DuplicateComponent(DuplicateComponentError),
}

impl From<EntityNotFoundError> for InsertBundleErrorKind {
    #[inline]
    fn from(error: EntityNotFoundError) -> Self {
        Self::EntityNotFound(error)
    }
}

impl From<DuplicateComponentError> for InsertBundleErrorKind {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl Display for InsertBundleErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityNotFound(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for InsertBundleErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EntityNotFound(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct InsertBundleError<B>
where
    B: Bundle,
{
    pub value: B,
    pub kind: InsertBundleErrorKind,
}

impl<B> From<ArchetypeInsertBundleError<B>> for InsertBundleError<B>
where
    B: Bundle,
{
    #[inline]
    fn from(error: ArchetypeInsertBundleError<B>) -> Self {
        let ArchetypeInsertBundleError { value, reason } = error;
        let kind = reason.into();
        Self { value, kind }
    }
}

impl<B> Display for InsertBundleError<B>
where
    B: Bundle + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, kind } = self;
        write!(f, "bundle {value} cannot be inserted: {kind}")
    }
}

impl<B> Error for InsertBundleError<B>
where
    B: Bundle + Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { kind, .. } = self;
        kind.source()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RemoveBundleError {
    EntityNotFound(EntityNotFoundError),
    DuplicateComponent(DuplicateComponentError),
}

impl From<EntityNotFoundError> for RemoveBundleError {
    #[inline]
    fn from(error: EntityNotFoundError) -> Self {
        Self::EntityNotFound(error)
    }
}

impl From<DuplicateComponentError> for RemoveBundleError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl Display for RemoveBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bundle cannot be removed: ")?;
        match self {
            Self::EntityNotFound(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for RemoveBundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EntityNotFound(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RemoveBundleExactError {
    EntityNotFound(EntityNotFoundError),
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
}

impl From<EntityNotFoundError> for RemoveBundleExactError {
    #[inline]
    fn from(error: EntityNotFoundError) -> Self {
        Self::EntityNotFound(error)
    }
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

impl From<ArchetypeRemoveBundleExactError> for RemoveBundleExactError {
    #[inline]
    fn from(error: ArchetypeRemoveBundleExactError) -> Self {
        match error {
            ArchetypeRemoveBundleExactError::DuplicateComponent(error) => {
                Self::DuplicateComponent(error)
            }
            ArchetypeRemoveBundleExactError::MissingComponent(error) => {
                Self::MissingComponent(error)
            }
        }
    }
}

impl Display for RemoveBundleExactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bundle cannot be removed: ")?;
        match self {
            Self::EntityNotFound(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for RemoveBundleExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EntityNotFound(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
        }
    }
}
