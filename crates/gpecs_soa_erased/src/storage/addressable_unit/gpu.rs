use super::{AddressableBy, AddressableUnit};

/// The guaranteed addressible unit for any GPU target.
impl AddressableUnit for u32 {}

/// ZSTs should be supported by any addressible unit.
unsafe impl AddressableBy<u32> for () {}

/// Supported on any GPU target.
unsafe impl AddressableBy<u32> for u32 {}
/// Supported on any GPU target.
unsafe impl AddressableBy<u32> for f32 {}
