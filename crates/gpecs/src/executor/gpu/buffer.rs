use std::ops::Range;

use wgpu::{BufferAddress, BufferSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferRegion {
    pub size: BufferSize,
    pub offset: BufferAddress,
}

impl From<BufferRegion> for Range<BufferAddress> {
    #[inline]
    fn from(region: BufferRegion) -> Self {
        let BufferRegion { size, offset } = region;
        let end = offset
            .checked_add(size.into())
            .expect("storage buffer region should be valid");
        offset..end
    }
}
