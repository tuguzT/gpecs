use core::alloc::Layout;

use crate::error::check_layout;

use super::error::{IntoValueError, SliceLenMismatchError};

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
pub fn check_into_layout<T, U>(layout: Layout, value: U) -> Result<U, IntoValueError<U>> {
    let expected = Layout::new::<T>();
    match check_layout(layout, expected) {
        Ok(()) => Ok(value),
        Err(reason) => Err(IntoValueError::new(value, reason)),
    }
}
