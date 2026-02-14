use std::{
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct NotRegisteredError;

impl Display for NotRegisteredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "component was not registered")
    }
}

impl Error for NotRegisteredError {}
