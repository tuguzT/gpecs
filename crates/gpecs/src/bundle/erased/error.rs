use std::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_soa_erased::{error::LayoutMismatchError, storage::AllocError};

use crate::{
    archetype::erased::error::{
        AlreadyHasComponentError, ArchetypeError, DuplicateComponentError,
        IncompatibleArchetypeError, IncompatibleArchetypeViewExactError, MissingComponentError,
    },
    component::erased::error::{
        ComponentMismatchError, DowncastErrorKind as ComponentDowncastErrorKind, NotRegisteredError,
    },
};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromBundleError<B> {
    pub source: FromBundleErrorKind,
    pub bundle: B,
}

impl<B> FromBundleError<B> {
    #[inline]
    pub(super) fn new(bundle: B, source: FromBundleErrorKind) -> Self {
        Self { source, bundle }
    }
}

impl<B> Display for FromBundleError<B>
where
    B: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, bundle } = self;
        write!(f, "failed to create erased bundle from {bundle}: {source}")
    }
}

impl<B> Error for FromBundleError<B>
where
    B: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum FromBundleErrorKind {
    DuplicateComponent(DuplicateComponentError),
    Alloc(AllocError),
}

impl From<DuplicateComponentError> for FromBundleErrorKind {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<AllocError> for FromBundleErrorKind {
    #[inline]
    fn from(error: AllocError) -> Self {
        Self::Alloc(error)
    }
}

impl Display for FromBundleErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::Alloc(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromBundleErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::Alloc(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FromComponentsError {
    Archetype(ArchetypeError),
    InvalidLayout(LayoutError),
    Alloc(AllocError),
}

impl From<ArchetypeError> for FromComponentsError {
    #[inline]
    fn from(error: ArchetypeError) -> Self {
        Self::Archetype(error)
    }
}

impl From<LayoutError> for FromComponentsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<AllocError> for FromComponentsError {
    #[inline]
    fn from(error: AllocError) -> Self {
        Self::Alloc(error)
    }
}

impl Display for FromComponentsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Archetype(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::Alloc(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromComponentsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Archetype(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::Alloc(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ShuffleError<T, A> {
    pub source: ShuffleErrorKind,
    pub bundle: T,
    pub archetype: A,
}

impl<T, A> From<ShuffleError<T, A>> for ShuffleErrorKind {
    #[inline]
    fn from(error: ShuffleError<T, A>) -> Self {
        let ShuffleError { source, .. } = error;
        source
    }
}

impl<T, A> Display for ShuffleError<T, A>
where
    T: Display,
    A: Display,
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

impl<T, A> Error for ShuffleError<T, A>
where
    T: Debug + Display,
    A: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum ShuffleErrorKind {
    IncompatibleArchetype(IncompatibleArchetypeViewExactError),
    InvalidLayout(LayoutError),
    Alloc(AllocError),
}

impl From<IncompatibleArchetypeViewExactError> for ShuffleErrorKind {
    #[inline]
    fn from(error: IncompatibleArchetypeViewExactError) -> Self {
        Self::IncompatibleArchetype(error)
    }
}

impl From<LayoutError> for ShuffleErrorKind {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<AllocError> for ShuffleErrorKind {
    #[inline]
    fn from(error: AllocError) -> Self {
        Self::Alloc(error)
    }
}

impl Display for ShuffleErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompatibleArchetype(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::Alloc(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ShuffleErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IncompatibleArchetype(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::Alloc(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsertError<T, I> {
    pub source: InsertErrorKind,
    pub bundle: T,
    pub to_insert: I,
}

impl<T, I> From<InsertError<T, I>> for InsertErrorKind {
    #[inline]
    fn from(error: InsertError<T, I>) -> Self {
        let InsertError { source, .. } = error;
        source
    }
}

impl<T, I> Display for InsertError<T, I>
where
    T: Display,
    I: Display,
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

impl<T, I> Error for InsertError<T, I>
where
    T: Debug + Display,
    I: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum InsertErrorKind {
    AlreadyHasComponent(AlreadyHasComponentError),
    InvalidLayout(LayoutError),
    Alloc(AllocError),
}

impl From<AlreadyHasComponentError> for InsertErrorKind {
    #[inline]
    fn from(error: AlreadyHasComponentError) -> Self {
        Self::AlreadyHasComponent(error)
    }
}

impl From<LayoutError> for InsertErrorKind {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<AllocError> for InsertErrorKind {
    #[inline]
    fn from(error: AllocError) -> Self {
        Self::Alloc(error)
    }
}

impl Display for InsertErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyHasComponent(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::Alloc(error) => Display::fmt(error, f),
        }
    }
}

impl Error for InsertErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AlreadyHasComponent(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::Alloc(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplaceError<T, R> {
    pub source: ReplaceErrorKind,
    pub bundle: T,
    pub to_replace: R,
}

impl<T, R> From<ReplaceError<T, R>> for ReplaceErrorKind {
    #[inline]
    fn from(error: ReplaceError<T, R>) -> Self {
        let ReplaceError { source, .. } = error;
        source
    }
}

impl<T, R> Display for ReplaceError<T, R>
where
    T: Display,
    R: Display,
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

impl<T, R> Error for ReplaceError<T, R>
where
    T: Debug + Display,
    R: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum ReplaceErrorKind {
    InvalidLayout(LayoutError),
    Alloc(AllocError),
}

impl From<LayoutError> for ReplaceErrorKind {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<AllocError> for ReplaceErrorKind {
    #[inline]
    fn from(error: AllocError) -> Self {
        Self::Alloc(error)
    }
}

impl Display for ReplaceErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::Alloc(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ReplaceErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::Alloc(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RemoveError<T> {
    pub source: RemoveErrorKind,
    pub bundle: T,
}

impl<T> From<RemoveError<T>> for RemoveErrorKind {
    #[inline]
    fn from(error: RemoveError<T>) -> Self {
        let RemoveError { source, .. } = error;
        source
    }
}

impl<T> Display for RemoveError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, bundle } = self;
        write!(f, "failed to remove components from {bundle}: {source}")
    }
}

impl<T> Error for RemoveError<T>
where
    T: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum RemoveErrorKind {
    MissingComponent(MissingComponentError),
    Alloc(AllocError),
}

impl From<MissingComponentError> for RemoveErrorKind {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<AllocError> for RemoveErrorKind {
    #[inline]
    fn from(error: AllocError) -> Self {
        Self::Alloc(error)
    }
}

impl Display for RemoveErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::Alloc(error) => Display::fmt(error, f),
        }
    }
}

impl Error for RemoveErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingComponent(error) => Some(error),
            Self::Alloc(error) => Some(error),
        }
    }
}

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
