use core::{mem::MaybeUninit, slice};

use crate::error::{LenMismatchError, check_len};

#[inline]
pub fn write_copy_of_slice<T>(dst: &mut [MaybeUninit<T>], src: &[T]) -> Result<(), LenMismatchError>
where
    T: Copy,
{
    let expected = dst.len();
    let len = src.len();
    check_len(len, expected)?;

    let src = unsafe { slice::from_raw_parts(src.as_ptr().cast(), len) };
    dst.copy_from_slice(src);

    Ok(())
}
