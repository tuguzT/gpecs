pub use self::{
    init::AlignedInitSlice,
    slice::{AlignedUninitSlice, AlignedUninitSliceError},
    traits::{AlignedSlice, AlignedSliceFromLayout},
};

#[cfg(feature = "alloc")]
pub use self::boxed::{AlignedUninitBoxedByteSlice, AllocError};

mod init;
mod slice;
mod traits;

#[cfg(feature = "alloc")]
mod boxed;
