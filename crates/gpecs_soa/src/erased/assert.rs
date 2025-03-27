use core::alloc::Layout;

use super::error::LenMismatchError;

#[cold]
#[track_caller]
#[inline(never)]
fn validate_layout_failed(input_align: usize, max_align: usize) -> ! {
    panic!("input alignment {input_align} must be less than or equal to {max_align}")
}

#[inline]
#[track_caller]
pub fn validate_layout<Fields>(layout: Layout) {
    let input_align = layout.align();
    let max_align = align_of::<Fields>();
    if input_align <= max_align {
        return;
    }
    validate_layout_failed(input_align, max_align)
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
