use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::{
    archetype::{
        erased::error::{
            AlreadyHasComponentError, DuplicateComponentError, IncompatibleArchetypeError,
            IncompatibleArchetypeExactError, MissingComponentError,
        },
        registry::ArchetypeId,
    },
    bundle::Bundle,
    component::erased::error::NotRegisteredError,
    entity::Entity,
};

#[derive(Debug, Clone)]
pub struct InvalidEntityLocationError {
    entity: Entity,
    archetype_id: ArchetypeId,
    kind: InvalidEntityLocationErrorKind,
}

impl InvalidEntityLocationError {
    #[inline]
    pub fn new(
        entity: Entity,
        archetype_id: ArchetypeId,
        kind: InvalidEntityLocationErrorKind,
    ) -> Self {
        Self {
            entity,
            archetype_id,
            kind,
        }
    }

    #[inline]
    pub fn entity(&self) -> Entity {
        let Self { entity, .. } = *self;
        entity
    }

    #[inline]
    pub fn archetype_id(&self) -> ArchetypeId {
        let Self { archetype_id, .. } = *self;
        archetype_id
    }

    #[inline]
    pub fn kind(&self) -> InvalidEntityLocationErrorKind {
        let Self { kind, .. } = *self;
        kind
    }

    #[cold]
    #[track_caller]
    #[inline(never)]
    pub(crate) fn with_valid_location(self) -> ! {
        unreachable!("{self}")
    }
}

impl Display for InvalidEntityLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            entity,
            archetype_id,
            kind,
        } = *self;

        match kind {
            InvalidEntityLocationErrorKind::UnknownArchetype => {
                write!(f, "unknown {archetype_id}")
            }
            InvalidEntityLocationErrorKind::EntityNotFound => {
                write!(f, "{archetype_id} should contain {entity}")
            }
            InvalidEntityLocationErrorKind::EntityHasComponents => {
                write!(f, "{archetype_id} should not contain {entity}")
            }
        }
    }
}

impl Error for InvalidEntityLocationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidEntityLocationErrorKind {
    UnknownArchetype,
    EntityNotFound,
    EntityHasComponents,
}

#[derive(Debug, Clone)]
pub enum GetAtError {
    InvalidEntityLocation(InvalidEntityLocationError),
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
    ComponentNotRegistered(NotRegisteredError),
}

impl GetAtError {
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> IncompatibleArchetypeError {
        match self {
            Self::InvalidEntityLocation(error) => error.with_valid_location(),
            Self::DuplicateComponent(error) => error.into(),
            Self::MissingComponent(error) => error.into(),
            Self::ComponentNotRegistered(error) => error.into(),
        }
    }
}

impl From<InvalidEntityLocationError> for GetAtError {
    #[inline]
    fn from(error: InvalidEntityLocationError) -> Self {
        Self::InvalidEntityLocation(error)
    }
}

impl From<DuplicateComponentError> for GetAtError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<MissingComponentError> for GetAtError {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<NotRegisteredError> for GetAtError {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::ComponentNotRegistered(error)
    }
}

impl From<IncompatibleArchetypeError> for GetAtError {
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

impl Display for GetAtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntityLocation(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
        }
    }
}

impl Error for GetAtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidEntityLocation(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
            Self::ComponentNotRegistered(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InsertExactError<T> {
    pub value: T,
    pub reason: AlreadyHasComponentError,
}

impl<T> Display for InsertExactError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, reason } = self;
        write!(f, "exact bundle {value} cannot be inserted: {reason}")
    }
}

impl<T> Error for InsertExactError<T>
where
    T: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Debug, Clone)]
pub enum InsertExactAtErrorKind {
    InvalidEntityLocation(InvalidEntityLocationError),
    AlreadyHasComponent(AlreadyHasComponentError),
}

impl InsertExactAtErrorKind {
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> AlreadyHasComponentError {
        match self {
            Self::InvalidEntityLocation(error) => error.with_valid_location(),
            Self::AlreadyHasComponent(error) => error,
        }
    }
}

impl From<InvalidEntityLocationError> for InsertExactAtErrorKind {
    #[inline]
    fn from(error: InvalidEntityLocationError) -> Self {
        Self::InvalidEntityLocation(error)
    }
}

impl From<AlreadyHasComponentError> for InsertExactAtErrorKind {
    #[inline]
    fn from(error: AlreadyHasComponentError) -> Self {
        Self::AlreadyHasComponent(error)
    }
}

impl Display for InsertExactAtErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntityLocation(error) => Display::fmt(error, f),
            Self::AlreadyHasComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for InsertExactAtErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidEntityLocation(error) => Some(error),
            Self::AlreadyHasComponent(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InsertExactAtError<T> {
    pub value: T,
    pub reason: InsertExactAtErrorKind,
}

impl<T> InsertExactAtError<T> {
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> InsertExactError<T> {
        let Self { value, reason } = self;

        let reason = reason.with_valid_location();
        InsertExactError { value, reason }
    }
}

impl<T> Display for InsertExactAtError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, reason } = self;
        write!(f, "exact bundle {value} cannot be inserted: {reason}")
    }
}

impl<T> Error for InsertExactAtError<T>
where
    T: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        reason.source()
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InsertAtError<T> {
    pub value: T,
    pub reason: InvalidEntityLocationError,
}

impl<T> InsertAtError<T> {
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> ! {
        let Self { reason, .. } = self;
        reason.with_valid_location()
    }
}

impl<T> Display for InsertAtError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, reason } = self;
        write!(f, "bundle {value} cannot be inserted: {reason}")
    }
}

impl<T> Error for InsertAtError<T>
where
    T: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InsertBundleExactErrorKind {
    DuplicateComponent(DuplicateComponentError),
    AlreadyHasComponent(AlreadyHasComponentError),
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
    pub reason: InsertBundleExactErrorKind,
}

impl<B> Display for InsertBundleExactError<B>
where
    B: Bundle + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, reason } = self;
        write!(f, "exact bundle {value} cannot be inserted: {reason}")
    }
}

impl<B> Error for InsertBundleExactError<B>
where
    B: Bundle + Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        reason.source()
    }
}

#[derive(Debug, Clone)]
pub enum InsertBundleExactAtErrorKind {
    InvalidEntityLocation(InvalidEntityLocationError),
    DuplicateComponent(DuplicateComponentError),
    AlreadyHasComponent(AlreadyHasComponentError),
}

impl InsertBundleExactAtErrorKind {
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> InsertBundleExactErrorKind {
        match self {
            Self::InvalidEntityLocation(error) => error.with_valid_location(),
            Self::DuplicateComponent(error) => error.into(),
            Self::AlreadyHasComponent(error) => error.into(),
        }
    }
}

impl From<InvalidEntityLocationError> for InsertBundleExactAtErrorKind {
    #[inline]
    fn from(error: InvalidEntityLocationError) -> Self {
        Self::InvalidEntityLocation(error)
    }
}

impl From<DuplicateComponentError> for InsertBundleExactAtErrorKind {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<AlreadyHasComponentError> for InsertBundleExactAtErrorKind {
    #[inline]
    fn from(error: AlreadyHasComponentError) -> Self {
        Self::AlreadyHasComponent(error)
    }
}

impl From<InsertExactAtErrorKind> for InsertBundleExactAtErrorKind {
    #[inline]
    fn from(error: InsertExactAtErrorKind) -> Self {
        use InsertExactAtErrorKind::{AlreadyHasComponent, InvalidEntityLocation};

        match error {
            InvalidEntityLocation(error) => Self::InvalidEntityLocation(error),
            AlreadyHasComponent(error) => Self::AlreadyHasComponent(error),
        }
    }
}

impl Display for InsertBundleExactAtErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntityLocation(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::AlreadyHasComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for InsertBundleExactAtErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidEntityLocation(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
            Self::AlreadyHasComponent(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InsertBundleExactAtError<B>
where
    B: Bundle,
{
    pub value: B,
    pub reason: InsertBundleExactAtErrorKind,
}

impl<B> InsertBundleExactAtError<B>
where
    B: Bundle,
{
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> InsertBundleExactError<B> {
        let Self { value, reason } = self;

        let reason = reason.with_valid_location();
        InsertBundleExactError { value, reason }
    }
}

impl<B> Display for InsertBundleExactAtError<B>
where
    B: Bundle + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, reason } = self;
        write!(f, "exact bundle {value} cannot be inserted: {reason}")
    }
}

impl<B> Error for InsertBundleExactAtError<B>
where
    B: Bundle + Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        reason.source()
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

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InsertBundleAtError<B>
where
    B: Bundle,
{
    pub value: B,
    pub reason: InsertBundleAtErrorKind,
}

impl<B> InsertBundleAtError<B>
where
    B: Bundle,
{
    #[inline]
    #[track_caller]
    pub(crate) fn into_insert_bundle_error(self) -> InsertBundleError<B> {
        let Self { value, reason } = self;

        let reason = reason.with_valid_location();
        InsertBundleError { value, reason }
    }
}

impl<B> Display for InsertBundleAtError<B>
where
    B: Bundle + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, reason } = self;
        write!(f, "bundle {value} cannot be inserted: {reason}")
    }
}

impl<B> Error for InsertBundleAtError<B>
where
    B: Bundle + Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        reason.source()
    }
}

#[derive(Debug, Clone)]
pub enum InsertBundleAtErrorKind {
    InvalidEntityLocation(InvalidEntityLocationError),
    DuplicateComponent(DuplicateComponentError),
}

impl InsertBundleAtErrorKind {
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> DuplicateComponentError {
        match self {
            Self::InvalidEntityLocation(error) => error.with_valid_location(),
            Self::DuplicateComponent(error) => error,
        }
    }
}

impl From<InvalidEntityLocationError> for InsertBundleAtErrorKind {
    #[inline]
    fn from(error: InvalidEntityLocationError) -> Self {
        Self::InvalidEntityLocation(error)
    }
}

impl From<DuplicateComponentError> for InsertBundleAtErrorKind {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl Display for InsertBundleAtErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntityLocation(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for InsertBundleAtErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidEntityLocation(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
        }
    }
}

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

#[derive(Debug, Clone)]
pub enum RemoveBundleExactAtError {
    InvalidEntityLocation(InvalidEntityLocationError),
    DuplicateComponent(DuplicateComponentError),
    MissingComponent(MissingComponentError),
}

impl RemoveBundleExactAtError {
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> RemoveBundleExactError {
        match self {
            Self::InvalidEntityLocation(error) => error.with_valid_location(),
            Self::DuplicateComponent(error) => error.into(),
            Self::MissingComponent(error) => error.into(),
        }
    }
}

impl From<InvalidEntityLocationError> for RemoveBundleExactAtError {
    #[inline]
    fn from(error: InvalidEntityLocationError) -> Self {
        Self::InvalidEntityLocation(error)
    }
}

impl From<DuplicateComponentError> for RemoveBundleExactAtError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl From<MissingComponentError> for RemoveBundleExactAtError {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl From<RemoveExactAtError> for RemoveBundleExactAtError {
    #[inline]
    fn from(error: RemoveExactAtError) -> Self {
        use RemoveExactAtError::{InvalidEntityLocation, MissingComponent};

        match error {
            InvalidEntityLocation(error) => Self::InvalidEntityLocation(error),
            MissingComponent(error) => Self::MissingComponent(error),
        }
    }
}

impl Display for RemoveBundleExactAtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exact bundle cannot be removed: ")?;
        match self {
            Self::InvalidEntityLocation(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for RemoveBundleExactAtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidEntityLocation(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RemoveExactAtError {
    InvalidEntityLocation(InvalidEntityLocationError),
    MissingComponent(MissingComponentError),
}

impl RemoveExactAtError {
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> MissingComponentError {
        match self {
            Self::InvalidEntityLocation(error) => error.with_valid_location(),
            Self::MissingComponent(error) => error,
        }
    }
}

impl From<InvalidEntityLocationError> for RemoveExactAtError {
    #[inline]
    fn from(error: InvalidEntityLocationError) -> Self {
        Self::InvalidEntityLocation(error)
    }
}

impl From<MissingComponentError> for RemoveExactAtError {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl Display for RemoveExactAtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntityLocation(error) => Display::fmt(error, f),
            Self::MissingComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for RemoveExactAtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidEntityLocation(error) => Some(error),
            Self::MissingComponent(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RemoveBundleAtError {
    InvalidEntityLocation(InvalidEntityLocationError),
    DuplicateComponent(DuplicateComponentError),
}

impl RemoveBundleAtError {
    #[inline]
    #[track_caller]
    pub(crate) fn with_valid_location(self) -> DuplicateComponentError {
        match self {
            Self::InvalidEntityLocation(error) => error.with_valid_location(),
            Self::DuplicateComponent(error) => error,
        }
    }
}

impl From<InvalidEntityLocationError> for RemoveBundleAtError {
    #[inline]
    fn from(error: InvalidEntityLocationError) -> Self {
        Self::InvalidEntityLocation(error)
    }
}

impl From<DuplicateComponentError> for RemoveBundleAtError {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl Display for RemoveBundleAtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntityLocation(error) => Display::fmt(error, f),
            Self::DuplicateComponent(error) => Display::fmt(error, f),
        }
    }
}

impl Error for RemoveBundleAtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidEntityLocation(error) => Some(error),
            Self::DuplicateComponent(error) => Some(error),
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
    pub reason: IncompatibleArchetypeExactError,
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
