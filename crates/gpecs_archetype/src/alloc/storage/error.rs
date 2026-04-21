use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_entity::Entity;

use crate::erased::error::{IncompatibleArchetypeExactError, IncompatibleArchetypeViewExactError};

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
pub struct EntityFoundError {
    entity: Entity,
}

impl EntityFoundError {
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

impl Display for EntityFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { entity } = self;
        write!(f, "{entity} was found")
    }
}

impl Error for EntityFoundError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MoveIntoError {
    IncompatibleArchetype(IncompatibleArchetypeViewExactError),
    SourceHasNoEntity(EntityNotFoundError),
    TargetHasEntity(EntityFoundError),
}

impl From<IncompatibleArchetypeViewExactError> for MoveIntoError {
    #[inline]
    fn from(error: IncompatibleArchetypeViewExactError) -> Self {
        Self::IncompatibleArchetype(error)
    }
}

impl From<EntityNotFoundError> for MoveIntoError {
    #[inline]
    fn from(error: EntityNotFoundError) -> Self {
        Self::SourceHasNoEntity(error)
    }
}

impl From<EntityFoundError> for MoveIntoError {
    #[inline]
    fn from(error: EntityFoundError) -> Self {
        Self::TargetHasEntity(error)
    }
}

impl Display for MoveIntoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompatibleArchetype(error) => Display::fmt(error, f),
            Self::SourceHasNoEntity(error) => Display::fmt(error, f),
            Self::TargetHasEntity(error) => Display::fmt(error, f),
        }
    }
}

impl Error for MoveIntoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IncompatibleArchetype(error) => Some(error),
            Self::SourceHasNoEntity(error) => Some(error),
            Self::TargetHasEntity(error) => Some(error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct IncompatibleBundleValueError<V>
where
    V: ?Sized,
{
    pub source: IncompatibleArchetypeExactError,
    pub value: V,
}

impl<V> IncompatibleBundleValueError<V> {
    #[inline]
    pub fn into_source(self) -> IncompatibleArchetypeExactError {
        let Self { source, .. } = self;
        source
    }
}

impl<V> Display for IncompatibleBundleValueError<V>
where
    V: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use IncompatibleArchetypeExactError::{
            ComponentNotRegistered, DuplicateComponent, MissingComponent, TooFewComponents,
        };

        let Self { value, source } = self;

        write!(f, "incompatible bundle {value}: ")?;
        match source {
            DuplicateComponent(error) => Display::fmt(error, f),
            MissingComponent(error) => Display::fmt(error, f),
            ComponentNotRegistered(error) => Display::fmt(error, f),
            TooFewComponents(error) => Display::fmt(error, f),
        }
    }
}

impl<V> Error for IncompatibleBundleValueError<V>
where
    V: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}
