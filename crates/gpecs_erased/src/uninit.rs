use core::{mem::MaybeUninit, slice};

use crate::error::{LenMismatchError, check_len};

#[inline]
pub fn try_init_copy_from_slice<T>(
    dst: &mut [MaybeUninit<T>],
    src: &[T],
) -> Result<(), LenMismatchError>
where
    T: Copy,
{
    let expected = dst.len();
    let len = src.len();
    check_len(len, expected)?;

    // FIXME: replace `unsafe` code below with regular `for` loop
    //        before that: find out the reason such a replacement generates UB
    //        inside of `ErasedSoaIntoFields` while running with `Miri`
    let src = unsafe { slice::from_raw_parts(src.as_ptr().cast(), len) };
    dst.copy_from_slice(src);

    Ok(())
}
