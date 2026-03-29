use core::alloc::{Layout, LayoutError};

/// [`Layout::repeat_packed()`], but on stable channel.
#[inline]
pub const fn repeat_packed(layout: Layout, n: usize) -> Result<Layout, LayoutError> {
    let size = layout.size().saturating_mul(n);
    Layout::from_size_align(size, layout.align())
}

/// [`Layout::repeat()`], but on stable channel.
#[inline]
pub const fn repeat(layout: Layout, n: usize) -> Result<(Layout, usize), LayoutError> {
    let padded = layout.pad_to_align();
    match repeat_packed(padded, n) {
        Ok(repeated) => Ok((repeated, padded.size())),
        Err(error) => Err(error),
    }
}
