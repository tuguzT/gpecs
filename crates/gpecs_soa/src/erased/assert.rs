use core::alloc::Layout;

use super::error::{InvalidLayoutError, LenMismatchError};

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

#[cold]
#[track_caller]
#[inline(never)]
fn assert_layouts_failed(first: Layout, second: Layout) -> ! {
    panic!("layouts {first:?} and {second:?} should match")
}

#[inline]
#[track_caller]
pub fn assert_layouts(first: Layout, second: Layout) {
    if first == second {
        return;
    }
    assert_layouts_failed(first, second)
}
