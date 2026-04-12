use std::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_soa_erased::error::InsufficientAlignError;

use crate::archetype::erased::error::{
    AlreadyHasComponentError, ArchetypeError, DuplicateComponentError,
    IncompatibleArchetypeViewExactError, MissingComponentError,
};

pub use gpecs_archetype::bundle::erased::error::{DowncastError, DowncastErrorKind};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromBundleError<B, T> {
    pub source: FromBundleErrorKind<T>,
    pub bundle: B,
}

impl<B, T> FromBundleError<B, T> {
    #[inline]
    pub(super) fn new(bundle: B, source: FromBundleErrorKind<T>) -> Self {
        Self { source, bundle }
    }
}

impl<B, T> Display for FromBundleError<B, T>
where
    B: Display,
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, bundle } = self;
        write!(f, "failed to create erased bundle from {bundle}: {source}")
    }
}

impl<B, T> Error for FromBundleError<B, T>
where
    B: Debug + Display,
    T: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum FromBundleErrorKind<T> {
    DuplicateComponent(DuplicateComponentError),
    InsufficientAlign(InsufficientAlignError),
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<InsufficientAlignError> for FromBundleErrorKind<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> From<LayoutError> for FromBundleErrorKind<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> From<DuplicateComponentError> for FromBundleErrorKind<T> {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl<T> Display for FromBundleErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromBundleErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FromComponentsError<T> {
    Archetype(ArchetypeError),
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<ArchetypeError> for FromComponentsError<T> {
    #[inline]
    fn from(error: ArchetypeError) -> Self {
        Self::Archetype(error)
    }
}

impl<T> From<LayoutError> for FromComponentsError<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> Display for FromComponentsError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Archetype(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromComponentsError<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Archetype(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::Archetype(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
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

#[derive(Debug, Clone)]
pub struct InsertError<T, I, E> {
    pub source: InsertErrorKind<E>,
    pub bundle: T,
    pub to_insert: I,
}

impl<T, I, E> From<InsertError<T, I, E>> for InsertErrorKind<E> {
    #[inline]
    fn from(error: InsertError<T, I, E>) -> Self {
        let InsertError { source, .. } = error;
        source
    }
}

impl<T, I, E> Display for InsertError<T, I, E>
where
    T: Display,
    I: Display,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            source,
            bundle,
            to_insert,
        } = self;

        write!(f, "failed to insert {to_insert} into {bundle}: {source}")
    }
}

impl<T, I, E> Error for InsertError<T, I, E>
where
    T: Debug + Display,
    I: Debug + Display,
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
pub enum InsertErrorKind<T> {
    AlreadyHasComponent(AlreadyHasComponentError),
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<AlreadyHasComponentError> for InsertErrorKind<T> {
    #[inline]
    fn from(error: AlreadyHasComponentError) -> Self {
        Self::AlreadyHasComponent(error)
    }
}

impl<T> From<LayoutError> for InsertErrorKind<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> Display for InsertErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyHasComponent(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for InsertErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AlreadyHasComponent(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::AlreadyHasComponent(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplaceError<T, R, E> {
    pub source: ReplaceErrorKind<E>,
    pub bundle: T,
    pub to_replace: R,
}

impl<T, R, E> From<ReplaceError<T, R, E>> for ReplaceErrorKind<E> {
    #[inline]
    fn from(error: ReplaceError<T, R, E>) -> Self {
        let ReplaceError { source, .. } = error;
        source
    }
}

impl<T, R, E> Display for ReplaceError<T, R, E>
where
    T: Display,
    R: Display,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            source,
            bundle,
            to_replace,
        } = self;

        write!(f, "failed to replace {to_replace} in {bundle}: {source}")
    }
}

impl<T, R, E> Error for ReplaceError<T, R, E>
where
    T: Debug + Display,
    R: Debug + Display,
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
pub enum ReplaceErrorKind<T> {
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<LayoutError> for ReplaceErrorKind<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> Display for ReplaceErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for ReplaceErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RemoveError<T, E> {
    pub source: RemoveErrorKind<E>,
    pub bundle: T,
}

impl<T, E> From<RemoveError<T, E>> for RemoveErrorKind<E> {
    #[inline]
    fn from(error: RemoveError<T, E>) -> Self {
        let RemoveError { source, .. } = error;
        source
    }
}

impl<T, E> Display for RemoveError<T, E>
where
    T: Display,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, bundle } = self;
        write!(f, "failed to remove components from {bundle}: {source}")
    }
}

impl<T, E> Error for RemoveError<T, E>
where
    T: Debug + Display,
    E: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum RemoveErrorKind<T> {
    MissingComponent(MissingComponentError),
    FromLayout(T),
}

impl<T> From<MissingComponentError> for RemoveErrorKind<T> {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl<T> Display for RemoveErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for RemoveErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingComponent(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::MissingComponent(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}
