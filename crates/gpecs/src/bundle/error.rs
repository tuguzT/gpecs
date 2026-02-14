use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_soa_erased::field::error::ErasedFieldIntoValueError;

#[derive(Debug, Clone)]
pub enum PtrsFromIterError<T> {
    IntoValue(ErasedFieldIntoValueError<T>),
    MissingComponent,
}

impl<T> From<ErasedFieldIntoValueError<T>> for PtrsFromIterError<T> {
    #[inline]
    fn from(error: ErasedFieldIntoValueError<T>) -> Self {
        Self::IntoValue(error)
    }
}

impl<T> Display for PtrsFromIterError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntoValue(error) => Display::fmt(error, f),
            Self::MissingComponent => write!(f, "required component is missing"),
        }
    }
}

impl<T> Error for PtrsFromIterError<T>
where
    T: Debug + Display,
{
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::IntoValue(error) => Some(error),
            Self::MissingComponent => None,
        }
    }
}
