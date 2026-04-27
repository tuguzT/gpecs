use core::{
    alloc::LayoutError,
    any,
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_erased::{
    data::error::{
        DowncastError as DataDowncastError, FromStorageError as DataFromStorageError,
        FromStorageErrorKind as DataFromStorageErrorKind, FromValueError, FromValueErrorKind,
        TryFromPtrError as DataTryFromPtrError, TryFromSlicePtrError as DataTryFromSlicePtrError,
    },
    error::{InsufficientAlignError, LayoutMismatchError, LenMismatchError, NotAlignedError},
};

use crate::{
    Component,
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentMismatchError {
    expected: ComponentId,
    actual: ComponentId,
}

impl ComponentMismatchError {
    #[inline]
    pub fn new(expected: ComponentId, actual: ComponentId) -> Option<Self> {
        if expected == actual {
            return None;
        }

        let me = unsafe { Self::new_unchecked(expected, actual) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(expected: ComponentId, actual: ComponentId) -> Self {
        Self { expected, actual }
    }

    #[inline]
    pub fn expected(&self) -> ComponentId {
        let Self { expected, .. } = *self;
        expected
    }

    #[inline]
    pub fn actual(&self) -> ComponentId {
        let Self { actual, .. } = *self;
        actual
    }
}

impl Display for ComponentMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = self;
        write!(f, "{actual} does not match expected {expected}")
    }
}

impl Error for ComponentMismatchError {}

#[inline]
pub fn check_component_ids(
    component_id: ComponentId,
    expected: ComponentId,
) -> Result<(), ComponentMismatchError> {
    ComponentMismatchError::new(expected, component_id).map_or(Ok(()), Err)
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct NotRegisteredError {
    name: Option<&'static str>,
}

impl NotRegisteredError {
    #[inline]
    pub const fn new() -> Self {
        Self { name: None }
    }

    #[inline]
    pub fn of<T>() -> Self
    where
        T: Component,
    {
        let name = any::type_name::<T>();
        Self { name: Some(name) }
    }
}

impl Display for NotRegisteredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { name } = self;

        match name {
            Some(name) => write!(f, "component `{name}` is not registered"),
            None => write!(f, "component is not registered"),
        }
    }
}

impl Error for NotRegisteredError {}

#[derive(Debug, Clone)]
pub enum DanglingError {
    NotRegistered(NotRegisteredError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotRegisteredError> for DanglingError {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::NotRegistered(error)
    }
}

impl From<InsufficientAlignError> for DanglingError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for DanglingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegistered(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for DanglingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotRegistered(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TryFromPtrError {
    NotRegistered(NotRegisteredError),
    NotAligned(NotAlignedError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotRegisteredError> for TryFromPtrError {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::NotRegistered(error)
    }
}

impl From<NotAlignedError> for TryFromPtrError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<InsufficientAlignError> for TryFromPtrError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl From<DataTryFromPtrError> for TryFromPtrError {
    #[inline]
    fn from(error: DataTryFromPtrError) -> Self {
        match error {
            DataTryFromPtrError::NotAligned(error) => Self::NotAligned(error),
            DataTryFromPtrError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

impl Display for TryFromPtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegistered(error) => Display::fmt(error, f),
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for TryFromPtrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotRegistered(error) => Some(error),
            Self::NotAligned(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TryFromSlicePtrError {
    NotRegistered(NotRegisteredError),
    InvalidLayout(LayoutError),
    NotAligned(NotAlignedError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotRegisteredError> for TryFromSlicePtrError {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::NotRegistered(error)
    }
}

impl From<LayoutError> for TryFromSlicePtrError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<NotAlignedError> for TryFromSlicePtrError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<InsufficientAlignError> for TryFromSlicePtrError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl From<DataTryFromSlicePtrError> for TryFromSlicePtrError {
    #[inline]
    fn from(error: DataTryFromSlicePtrError) -> Self {
        match error {
            DataTryFromSlicePtrError::InvalidLayout(error) => Self::InvalidLayout(error),
            DataTryFromSlicePtrError::NotAligned(error) => Self::NotAligned(error),
            DataTryFromSlicePtrError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

impl Display for TryFromSlicePtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegistered(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for TryFromSlicePtrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotRegistered(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::NotAligned(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DowncastErrorKind {
    ComponentNotRegistered(NotRegisteredError),
    ComponentMismatch(ComponentMismatchError),
    LayoutMismatch(LayoutMismatchError),
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

impl Display for DowncastErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
            Self::ComponentMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for DowncastErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentNotRegistered(error) => Some(error),
            Self::ComponentMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
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

impl<T> From<DataDowncastError<T>> for DowncastError<T> {
    #[inline]
    fn from(error: DataDowncastError<T>) -> Self {
        let DataDowncastError { source, value, .. } = error;
        let source = source.into();
        Self::new(value, source)
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
        write!(f, "failed to downcast {value} into component: {source}")
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

#[inline]
pub(super) fn check_downcast<C, T, V>(
    components: &ComponentRegistryView<impl Sized, T>,
    component_id: ComponentId,
    value: V,
) -> Result<V, DowncastError<V>>
where
    C: Component,
    T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
{
    match check_downcast_inner::<C, T>(components, component_id) {
        Ok(()) => Ok(value),
        Err(source) => Err(DowncastError::new(value, source)),
    }
}

#[inline]
fn check_downcast_inner<C, T>(
    components: &ComponentRegistryView<impl Sized, T>,
    component_id: ComponentId,
) -> Result<(), DowncastErrorKind>
where
    C: Component,
    T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
{
    let into_component_id = components
        .component_id::<C>()
        .ok_or_else(NotRegisteredError::of::<C>)?;
    check_component_ids(into_component_id, component_id)?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum FromComponentErrorKind<T> {
    NotRegistered(NotRegisteredError),
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T),
}

impl<T> From<NotRegisteredError> for FromComponentErrorKind<T> {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::NotRegistered(error)
    }
}

impl<T> From<InsufficientAlignError> for FromComponentErrorKind<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> From<FromValueErrorKind<T>> for FromComponentErrorKind<T> {
    #[inline]
    fn from(error: FromValueErrorKind<T>) -> Self {
        match error {
            FromValueErrorKind::InsufficientAlign(error) => Self::InsufficientAlign(error),
            FromValueErrorKind::FromLayout(error) => Self::FromLayout(error),
        }
    }
}

impl<T> Display for FromComponentErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegistered(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromComponentErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotRegistered(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::NotRegistered(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromComponentError<T, C> {
    pub source: FromComponentErrorKind<T>,
    pub component: C,
}

impl<T, C> FromComponentError<T, C> {
    #[inline]
    pub(super) fn new(component: C, source: FromComponentErrorKind<T>) -> Self {
        Self { source, component }
    }
}

impl<T, C> From<FromValueError<T, C>> for FromComponentError<T, C> {
    #[inline]
    fn from(error: FromValueError<T, C>) -> Self {
        let FromValueError { value, source, .. } = error;
        Self::new(value, source.into())
    }
}

impl<T, C> Display for FromComponentError<T, C>
where
    T: Display,
    C: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, component } = self;
        write!(f, "failed to convert component {component}: {source}")
    }
}

impl<T, C> Error for FromComponentError<T, C>
where
    T: Error,
    C: Debug + Display,
{
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromStorageError<T>
where
    T: ?Sized,
{
    pub source: FromStorageErrorKind,
    pub storage: T,
}

impl<T> FromStorageError<T> {
    pub(crate) fn new(source: FromStorageErrorKind, storage: T) -> Self {
        Self { source, storage }
    }
}

impl<T> From<DataFromStorageError<T>> for FromStorageError<T> {
    #[inline]
    fn from(error: DataFromStorageError<T>) -> Self {
        let DataFromStorageError {
            source, storage, ..
        } = error;
        let source = source.into();
        Self::new(source, storage)
    }
}

impl<T> Display for FromStorageError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, storage } = self;
        write!(
            f,
            "failed to create erased component with {storage}: {source}"
        )
    }
}

impl<T> Error for FromStorageError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum FromStorageErrorKind {
    NotRegistered(NotRegisteredError),
    NotAligned(NotAlignedError),
    LenMismatch(LenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<DataFromStorageErrorKind> for FromStorageErrorKind {
    #[inline]
    fn from(error: DataFromStorageErrorKind) -> Self {
        match error {
            DataFromStorageErrorKind::NotAligned(error) => Self::NotAligned(error),
            DataFromStorageErrorKind::LenMismatch(error) => Self::LenMismatch(error),
            DataFromStorageErrorKind::LayoutMismatch(error) => Self::LayoutMismatch(error),
            DataFromStorageErrorKind::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

impl From<NotRegisteredError> for FromStorageErrorKind {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::NotRegistered(error)
    }
}

impl From<NotAlignedError> for FromStorageErrorKind {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LenMismatchError> for FromStorageErrorKind {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for FromStorageErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<InsufficientAlignError> for FromStorageErrorKind {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for FromStorageErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegistered(error) => Display::fmt(error, f),
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromStorageErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotRegistered(error) => Some(error),
            Self::NotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}
