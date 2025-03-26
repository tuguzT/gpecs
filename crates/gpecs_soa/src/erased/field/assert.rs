use core::alloc::Layout;

use super::{error::LayoutMismatchError, PtrNotAlignedError};

#[cold]
#[track_caller]
#[inline(never)]
fn assert_value_buffer_len_failed(buffer_len: usize, layout_size: usize) -> ! {
    panic!("buffer len {buffer_len} should match layout size {layout_size}")
}

#[inline]
#[track_caller]
pub fn assert_value_buffer_len(buffer_len: usize, layout_size: usize) {
    if buffer_len == layout_size {
        return;
    }
    assert_value_buffer_len_failed(buffer_len, layout_size)
}

#[cold]
#[track_caller]
#[inline(never)]
fn assert_slice_buffer_len_failed(buffer_len: usize, layout_size: usize, len: usize) -> ! {
    panic!("buffer len {buffer_len} divided by layout size {layout_size} should be equal to {len}")
}

#[inline]
#[track_caller]
pub fn assert_slice_buffer_len(buffer_len: usize, layout_size: usize, len: usize) {
    if layout_size == 0 && buffer_len == 0 {
        return;
    }
    if buffer_len.div_ceil(layout_size) == len {
        return;
    }
    assert_slice_buffer_len_failed(buffer_len, layout_size, len)
}

#[inline]
pub fn check_buffer_align(ptr: *const u8, target_layout: Layout) -> Result<(), PtrNotAlignedError> {
    match ptr.align_offset(target_layout.align()) {
        0 => Ok(()),
        _ => Err(PtrNotAlignedError::new(ptr, target_layout)),
    }
}

#[inline]
pub fn check_layout<T, U>(layout: Layout, value: U) -> Result<U, LayoutMismatchError<U>> {
    let expected = Layout::new::<T>();
    match layout == expected {
        true => Ok(value),
        false => Err(LayoutMismatchError::new(value, expected, layout)),
    }
}
