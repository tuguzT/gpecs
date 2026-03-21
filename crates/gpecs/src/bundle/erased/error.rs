use std::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_soa_erased::storage::AllocError;

use crate::archetype::error::{
    AlreadyHasComponentError, DuplicateComponentError, IncompatibleArchetypeError,
    IncompatibleArchetypeExactError, MissingComponentError,
};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromBundleError<B> {
    pub reason: FromBundleErrorKind,
    pub bundle: B,
}

impl<B> FromBundleError<B> {
    #[inline]
    pub(super) fn new(bundle: B, reason: FromBundleErrorKind) -> Self {
        Self { reason, bundle }
    }
}

impl<B> Display for FromBundleError<B>
where
    B: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, bundle } = self;
        write!(f, "failed to create erased bundle from {bundle}: {reason}")
    }
}

impl<B> Error for FromBundleError<B>
where
    B: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
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
    DuplicateComponent(DuplicateComponentError),
    InvalidLayout(LayoutError),
    Alloc(AllocError),
}

impl From<DuplicateComponentError> for FromComponentsError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
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
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::Alloc(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromComponentsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::Alloc(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ShuffleError<T, A> {
    pub reason: ShuffleErrorKind,
    pub bundle: T,
    pub archetype: A,
}

impl<T, A> From<ShuffleError<T, A>> for ShuffleErrorKind {
    #[inline]
    fn from(error: ShuffleError<T, A>) -> Self {
        let ShuffleError { reason, .. } = error;
        reason
    }
}

impl<T, A> Display for ShuffleError<T, A>
where
    T: Display,
    A: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            reason,
            bundle,
            archetype,
        } = self;

        write!(f, "failed to shuffle {bundle} by {archetype}: {reason}")
    }
}

impl<T, A> Error for ShuffleError<T, A>
where
    T: Debug + Display,
    A: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Debug, Clone)]
pub enum ShuffleErrorKind {
    IncompatibleArchetype(IncompatibleArchetypeExactError),
    InvalidLayout(LayoutError),
    Alloc(AllocError),
}

impl From<IncompatibleArchetypeExactError> for ShuffleErrorKind {
    #[inline]
    fn from(error: IncompatibleArchetypeExactError) -> Self {
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
    pub reason: InsertErrorKind,
    pub bundle: T,
    pub to_insert: I,
}

impl<T, I> From<InsertError<T, I>> for InsertErrorKind {
    #[inline]
    fn from(error: InsertError<T, I>) -> Self {
        let InsertError { reason, .. } = error;
        reason
    }
}

impl<T, I> Display for InsertError<T, I>
where
    T: Display,
    I: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            reason,
            bundle,
            to_insert,
        } = self;

        write!(f, "failed to insert {to_insert} into {bundle}: {reason}")
    }
}

impl<T, I> Error for InsertError<T, I>
where
    T: Debug + Display,
    I: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
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
    pub reason: ReplaceErrorKind,
    pub bundle: T,
    pub to_replace: R,
}

impl<T, R> From<ReplaceError<T, R>> for ReplaceErrorKind {
    #[inline]
    fn from(error: ReplaceError<T, R>) -> Self {
        let ReplaceError { reason, .. } = error;
        reason
    }
}

impl<T, R> Display for ReplaceError<T, R>
where
    T: Display,
    R: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            reason,
            bundle,
            to_replace,
        } = self;

        write!(f, "failed to replace {to_replace} in {bundle}: {reason}")
    }
}

impl<T, R> Error for ReplaceError<T, R>
where
    T: Debug + Display,
    R: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
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
    pub reason: RemoveErrorKind,
    pub bundle: T,
}

impl<T> From<RemoveError<T>> for RemoveErrorKind {
    #[inline]
    fn from(error: RemoveError<T>) -> Self {
        let RemoveError { reason, .. } = error;
        reason
    }
}

impl<T> Display for RemoveError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, bundle } = self;
        write!(f, "failed to remove components from {bundle}: {reason}")
    }
}

impl<T> Error for RemoveError<T>
where
    T: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
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
    pub reason: IncompatibleArchetypeError,
    pub value: T,
}

impl<T> DowncastError<T> {
    #[inline]
    pub(super) fn new(value: T, reason: IncompatibleArchetypeError) -> Self {
        Self { reason, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> DowncastError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { reason, value } = self;
        DowncastError::new(f(value), reason)
    }
}

impl<T> From<DowncastError<T>> for IncompatibleArchetypeError {
    #[inline]
    fn from(error: DowncastError<T>) -> Self {
        let DowncastError { reason, .. } = error;
        reason
    }
}

impl<T> Display for DowncastError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to downcast {value} into component: {reason}")
    }
}

impl<T> Error for DowncastError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}
