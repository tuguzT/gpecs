use core::alloc::Layout;

use super::error::{InvalidLayoutError, LayoutMismatchError, LenMismatchError};

#[inline]
pub fn validate_layout<Fields>(layout: Layout) -> Result<(), InvalidLayoutError> {
    let max_align = Layout::new::<Fields>();
    if layout.align() > max_align.align() {
        return Err(InvalidLayoutError::new(layout, max_align));
    }
    Ok(())
}

#[inline]
pub fn check_same_len(len: usize, expected_len: usize) -> Result<(), LenMismatchError> {
    if len != expected_len {
        return Err(LenMismatchError::new(expected_len, len));
    }
    Ok(())
}

#[inline]
pub fn check_same_layout(layout: Layout, expected: Layout) -> Result<(), LayoutMismatchError> {
    if layout != expected {
        return Err(LayoutMismatchError::new(expected, layout));
    }
    Ok(())
}
