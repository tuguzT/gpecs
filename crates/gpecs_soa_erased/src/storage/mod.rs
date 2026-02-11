pub use self::{
    init::AlignedInitStorage,
    slice::{AlignedUninitStorage, AlignedUninitStorageError},
    traits::{AlignedStorage, AlignedStorageFromLayout},
};

#[cfg(feature = "alloc")]
pub use self::boxed::{AllocError, BoxedAlignedUninitStorage};

mod init;
mod slice;
mod traits;

#[cfg(feature = "alloc")]
mod boxed;
