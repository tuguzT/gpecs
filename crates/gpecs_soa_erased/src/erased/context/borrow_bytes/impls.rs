use alloc::{collections::TryReserveError, vec::Vec};
use core::{
    cell::{BorrowMutError, RefCell, RefMut},
    error::Error,
    fmt::{self, Display},
    mem::MaybeUninit,
};

use crate::error::{LenMismatchError, check_len};

use super::BorrowBytes;

#[derive(Debug)]
pub enum RefCellUninitByteSliceError {
    LenMismatch(LenMismatchError),
    RefCellBorrowMut(BorrowMutError),
}

impl From<LenMismatchError> for RefCellUninitByteSliceError {
    #[inline]
    fn from(err: LenMismatchError) -> Self {
        Self::LenMismatch(err)
    }
}

impl From<BorrowMutError> for RefCellUninitByteSliceError {
    #[inline]
    fn from(err: BorrowMutError) -> Self {
        Self::RefCellBorrowMut(err)
    }
}

impl Display for RefCellUninitByteSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(err) => Display::fmt(err, f),
            Self::RefCellBorrowMut(err) => Display::fmt(err, f),
        }
    }
}

impl Error for RefCellUninitByteSliceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(err) => Some(err),
            Self::RefCellBorrowMut(err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub enum RefCellByteVecError {
    RefCellBorrowMut(BorrowMutError),
    TryReserve(TryReserveError),
}

impl From<BorrowMutError> for RefCellByteVecError {
    #[inline]
    fn from(err: BorrowMutError) -> Self {
        Self::RefCellBorrowMut(err)
    }
}

impl From<TryReserveError> for RefCellByteVecError {
    #[inline]
    fn from(err: TryReserveError) -> Self {
        Self::TryReserve(err)
    }
}

impl Display for RefCellByteVecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RefCellBorrowMut(err) => Display::fmt(err, f),
            Self::TryReserve(err) => Display::fmt(err, f),
        }
    }
}

impl Error for RefCellByteVecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RefCellBorrowMut(err) => Some(err),
            Self::TryReserve(err) => Some(err),
        }
    }
}

impl BorrowBytes for RefCell<[MaybeUninit<u8>]> {
    type Output<'a>
        = RefMut<'a, [MaybeUninit<u8>]>
    where
        Self: 'a;

    type Error = RefCellUninitByteSliceError;

    fn borrow_bytes(&self, count: usize) -> Result<Self::Output<'_>, Self::Error> {
        let mut bytes = self.try_borrow_mut()?;
        check_len(bytes.as_mut().len(), count)?;

        let bytes = RefMut::map(bytes, |bytes| {
            let bytes = bytes.as_mut();
            &mut bytes[..count]
        });
        Ok(bytes)
    }
}

impl BorrowBytes for RefCell<Vec<u8>> {
    type Output<'a>
        = RefMut<'a, [MaybeUninit<u8>]>
    where
        Self: 'a;

    type Error = RefCellByteVecError;

    fn borrow_bytes(&self, count: usize) -> Result<Self::Output<'_>, Self::Error> {
        let mut bytes = self.try_borrow_mut()?;
        bytes.clear();
        bytes.try_reserve(count)?;

        let bytes = RefMut::map(bytes, |bytes| {
            let bytes = bytes.spare_capacity_mut();
            &mut bytes[..count]
        });
        Ok(bytes)
    }
}
