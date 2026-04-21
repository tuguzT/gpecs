use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::{
    archetype::{
        erased::error::{AlreadyHasComponentError, DuplicateComponentError, MissingComponentError},
        registry::error::{
            InsertBundleError as ArchetypeInsertBundleError,
            InsertBundleExactError as ArchetypeInsertBundleExactError,
            InsertBundleExactErrorKind as ArchetypeInsertBundleExactErrorKind,
            RemoveBundleExactError as ArchetypeRemoveBundleExactError,
        },
        storage::error::EntityNotFoundError,
    },
    bundle::{Bundle, erased::error::DowncastErrorKind},
    entity::Entity,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EntityHasNoDataError {
    entity: Entity,
}

impl EntityHasNoDataError {
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

impl Display for EntityHasNoDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { entity } = self;
        write!(f, "{entity} has no data attached to it")
    }
}

impl Error for EntityHasNoDataError {}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum IncompatibleBundleError {
    EntityNotFound(EntityNotFoundError),
    EntityHasNoData(EntityHasNoDataError),
    Downcast(DowncastErrorKind),
}

impl From<EntityNotFoundError> for IncompatibleBundleError {
    #[inline]
    fn from(error: EntityNotFoundError) -> Self {
        Self::EntityNotFound(error)
    }
}

impl From<EntityHasNoDataError> for IncompatibleBundleError {
    #[inline]
    fn from(error: EntityHasNoDataError) -> Self {
        Self::EntityHasNoData(error)
    }
}

impl From<DowncastErrorKind> for IncompatibleBundleError {
    #[inline]
    fn from(error: DowncastErrorKind) -> Self {
        Self::Downcast(error)
    }
}

impl Display for IncompatibleBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incompatible bundle: ")?;
        match self {
            Self::EntityNotFound(error) => Display::fmt(error, f),
            Self::EntityHasNoData(error) => Display::fmt(error, f),
            Self::Downcast(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IncompatibleBundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EntityNotFound(error) => Some(error),
            Self::EntityHasNoData(error) => Some(error),
            Self::Downcast(error) => Some(error),
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
    pub source: InsertBundleExactErrorKind,
}

impl<B> From<ArchetypeInsertBundleExactError<B>> for InsertBundleExactError<B>
where
    B: Bundle,
{
    #[inline]
    fn from(error: ArchetypeInsertBundleExactError<B>) -> Self {
        let ArchetypeInsertBundleExactError { value, source } = error;
        let source = source.into();
        Self { value, source }
    }
}

impl<B> Display for InsertBundleExactError<B>
where
    B: Bundle + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, source } = self;
        write!(f, "exact bundle {value} cannot be inserted: {source}")
    }
}

impl<B> Error for InsertBundleExactError<B>
where
    B: Bundle + Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        source.source()
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
    pub source: InsertBundleErrorKind,
}

impl<B> From<ArchetypeInsertBundleError<B>> for InsertBundleError<B>
where
    B: Bundle,
{
    #[inline]
    fn from(error: ArchetypeInsertBundleError<B>) -> Self {
        let ArchetypeInsertBundleError { value, source } = error;
        let source = source.into();
        Self { value, source }
    }
}

impl<B> Display for InsertBundleError<B>
where
    B: Bundle + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, source } = self;
        write!(f, "bundle {value} cannot be inserted: {source}")
    }
}

impl<B> Error for InsertBundleError<B>
where
    B: Bundle + Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        source.source()
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
    EntityHasNoData(EntityHasNoDataError),
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
}

impl From<EntityNotFoundError> for RemoveBundleExactError {
    #[inline]
    fn from(error: EntityNotFoundError) -> Self {
        Self::EntityNotFound(error)
    }
}

impl From<EntityHasNoDataError> for RemoveBundleExactError {
    #[inline]
    fn from(error: EntityHasNoDataError) -> Self {
        Self::EntityHasNoData(error)
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
            Self::EntityHasNoData(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for RemoveBundleExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EntityNotFound(error) => Some(error),
            Self::EntityHasNoData(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
        }
    }
}
