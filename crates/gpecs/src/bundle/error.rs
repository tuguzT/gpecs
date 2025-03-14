use std::{
    error::Error,
    fmt::{self, Display},
};

use crate::component::registry::ComponentId;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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
