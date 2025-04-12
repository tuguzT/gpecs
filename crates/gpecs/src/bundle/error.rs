use std::{
    any,
    borrow::Cow,
    error::Error,
    fmt::{self, Display},
};

use crate::component::{registry::ComponentId, Component};

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
        write!(f, "duplicate component {component_id:?} were found")
    }
}

impl Error for DuplicateComponentError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ComponentNotRegisteredError {
    name: Option<Cow<'static, str>>,
}

impl ComponentNotRegisteredError {
    #[inline]
    pub fn new() -> Self {
        Self { name: None }
    }

    #[inline]
    pub fn with_name(name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        Self { name: Some(name) }
    }

    #[inline]
    pub fn of<C>() -> Self
    where
        C: Component,
    {
        let name = any::type_name::<C>();
        Self::with_name(name)
    }
}

impl Display for ComponentNotRegisteredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { name } = self;

        write!(f, "component ")?;
        if let Some(name) = name {
            write!(f, "{name:?} ")?;
        }
        write!(f, "was not registered")
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
