use super::{AddressableBy, AddressableUnit};

/// The smallest addressible unit for any CPU target.
impl AddressableUnit for u8 {}

/// Any Rust type is aligned to [`u8`], which size is exactly 1 byte.
unsafe impl<T> AddressableBy<u8> for T where T: ?Sized {}
