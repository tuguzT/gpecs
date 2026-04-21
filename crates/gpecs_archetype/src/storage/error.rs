#[cfg(feature = "alloc")]
pub use crate::alloc::storage::error::{
    EntityFoundError, EntityNotFoundError, IncompatibleBundleValueError, MoveIntoError,
    UpdateWithError, UpdateWithErrorKind,
};
