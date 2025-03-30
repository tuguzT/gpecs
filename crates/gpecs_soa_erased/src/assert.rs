use core::alloc::Layout;

use crate::error::{LayoutMismatchError, LenMismatchError};

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
