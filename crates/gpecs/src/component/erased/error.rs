use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_soa_erased::storage::AllocError;

use crate::component::{
    Component,
    error::NotRegisteredError,
    registry::{ComponentId, ComponentRegistry},
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum DowncastErrorKind {
    ComponentNotRegistered(NotRegisteredError),
    ComponentMismatch(ComponentMismatchError),
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

impl Display for DowncastErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ComponentNotRegistered(error) => Display::fmt(error, f),
            Self::ComponentMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for DowncastErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentNotRegistered(error) => Some(error),
            Self::ComponentMismatch(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DowncastError<T>
where
    T: ?Sized,
{
    pub reason: DowncastErrorKind,
    pub value: T,
}

impl<T> DowncastError<T> {
    #[inline]
    pub fn new(value: T, reason: DowncastErrorKind) -> Self {
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

impl<T> From<DowncastError<T>> for DowncastErrorKind {
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

#[inline]
pub(super) fn check_downcast<C, T>(
    registry: &ComponentRegistry,
    component_id: ComponentId,
    value: T,
) -> Result<T, DowncastError<T>>
where
    C: Component,
{
    match check_downcast_inner::<C>(registry, component_id) {
        Ok(()) => Ok(value),
        Err(reason) => Err(DowncastError::new(value, reason)),
    }
}

#[inline]
fn check_downcast_inner<C>(
    registry: &ComponentRegistry,
    component_id: ComponentId,
) -> Result<(), DowncastErrorKind>
where
    C: Component,
{
    let into_component_id = registry.component_id::<C>().ok_or(NotRegisteredError)?;
    check_component_ids(into_component_id, component_id)?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromComponentErrorKind {
    NotRegistered(NotRegisteredError),
    Alloc(AllocError),
}

impl From<NotRegisteredError> for FromComponentErrorKind {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::NotRegistered(error)
    }
}

impl From<AllocError> for FromComponentErrorKind {
    #[inline]
    fn from(error: AllocError) -> Self {
        Self::Alloc(error)
    }
}

impl Display for FromComponentErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegistered(error) => Display::fmt(error, f),
            Self::Alloc(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromComponentErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotRegistered(error) => Some(error),
            Self::Alloc(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromComponentError<C> {
    pub reason: FromComponentErrorKind,
    pub component: C,
}

impl<C> FromComponentError<C> {
    #[inline]
    pub(super) fn new(component: C, reason: FromComponentErrorKind) -> Self {
        Self { reason, component }
    }
}

impl<C> Display for FromComponentError<C>
where
    C: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, component } = self;
        write!(f, "failed to convert component {component}: {reason}")
    }
}

impl<C> Error for FromComponentError<C>
where
    C: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}
