#[cfg(feature = "alloc")]
pub use crate::alloc::storage::error::{
    EntityFoundError, EntityNotFoundError, IncompatibleBundleValueError, MoveIntoError,
    MoveIntoWithInsertBundleError, MoveIntoWithInsertBundleErrorKind, MoveIntoWithInsertError,
    MoveIntoWithInsertErrorKind, UpdateWithBundleError, UpdateWithBundleErrorKind, UpdateWithError,
    UpdateWithErrorKind,
};
