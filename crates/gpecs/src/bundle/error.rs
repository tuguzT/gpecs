use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_soa_erased::field::error::DowncastError;

use crate::component::error::NotRegisteredError;

#[derive(Debug, Clone)]
pub enum PtrsFromIterError<T> {
    Downcast(DowncastError<T>),
    NotRegistered(NotRegisteredError),
}

impl<T> From<DowncastError<T>> for PtrsFromIterError<T> {
    #[inline]
    fn from(error: DowncastError<T>) -> Self {
        Self::Downcast(error)
    }
}

impl<T> From<NotRegisteredError> for PtrsFromIterError<T> {
    #[inline]
    fn from(error: NotRegisteredError) -> Self {
        Self::NotRegistered(error)
    }
}

impl<T> Display for PtrsFromIterError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Downcast(error) => Display::fmt(error, f),
            Self::NotRegistered(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for PtrsFromIterError<T>
where
    T: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Downcast(_) => None,
            Self::NotRegistered(error) => Some(error),
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::Downcast(error) => Some(error),
            Self::NotRegistered(error) => Some(error),
        }
    }
}
