use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};
use core::mem::MaybeUninit;

use super::BorrowBytes;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NewBytes;

impl BorrowBytes for NewBytes {
    type Output<'a>
        = Box<[MaybeUninit<u8>]>
    where
        Self: 'a;

    type Error = TryReserveError;

    #[inline]
    fn borrow_bytes(&self, count: usize) -> Result<Self::Output<'_>, Self::Error> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(count)?;
        unsafe {
            bytes.set_len(count);
        }

        let bytes = bytes.into_boxed_slice();
        Ok(bytes)
    }
}
