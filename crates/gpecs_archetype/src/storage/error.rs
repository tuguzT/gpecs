#[cfg(feature = "alloc")]
pub use crate::alloc::storage::error::{
    EntityFoundError, EntityNotFoundError, IncompatibleBundleValueError, MoveIntoError,
    UpdateWithBundleError, UpdateWithBundleErrorKind, UpdateWithError, UpdateWithErrorKind,
};
