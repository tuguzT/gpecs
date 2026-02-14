use std::{
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct ComponentNotRegisteredError;

impl Display for ComponentNotRegisteredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "component was not registered")
    }
}

impl Error for ComponentNotRegisteredError {}
