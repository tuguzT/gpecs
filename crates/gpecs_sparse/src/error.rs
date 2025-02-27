use alloc::collections::TryReserveError as AllocError;
use core::{
    error::Error,
    fmt::{self, Display},
};

use crate::soa::vec::TryReserveError as SoaError;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TryReserveError {
    Sparse(AllocError),
    Dense(SoaError),
}

impl From<AllocError> for TryReserveError {
    fn from(v: AllocError) -> Self {
        Self::Sparse(v)
    }
}

impl From<SoaError> for TryReserveError {
    fn from(v: SoaError) -> Self {
        Self::Dense(v)
    }
}

impl Display for TryReserveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sparse(e) => write!(f, "sparse: {}", e),
            Self::Dense(e) => write!(f, "dense: {}", e),
        }
    }
}

impl Error for TryReserveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sparse(e) => Some(e),
            Self::Dense(e) => Some(e),
        }
    }
}
