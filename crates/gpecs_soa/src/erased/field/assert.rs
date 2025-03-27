use core::alloc::Layout;

use super::error::{LayoutMismatchError, PtrNotAlignedError, SliceLenMismatchError};

#[inline]
pub fn check_slice_buffer_len(
    buffer_len: usize,
    layout_size: usize,
    len: usize,
) -> Result<(), SliceLenMismatchError> {
    if layout_size == 0 {
        return match buffer_len {
            0 => Ok(()),
            _ => Err(SliceLenMismatchError::new(0, len, buffer_len)),
        };
    }

    if buffer_len.div_ceil(layout_size) != len {
        return Err(SliceLenMismatchError::new(layout_size, len, buffer_len));
    }
    Ok(())
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
    if layout != expected {
        return Err(LayoutMismatchError::new(value, expected, layout));
    }
    Ok(value)
}
