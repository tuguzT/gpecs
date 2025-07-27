use core::{mem::MaybeUninit, ops::DerefMut};

pub use self::{
    impls::{RefCellByteVecError, RefCellUninitByteSliceError},
    new_bytes::NewBytes,
};

mod impls;
mod new_bytes;

pub trait BorrowBytes {
    type Output<'a>: DerefMut<Target = [MaybeUninit<u8>]>
    where
        Self: 'a;

    type Error;

    fn borrow_bytes(&self, count: usize) -> Result<Self::Output<'_>, Self::Error>;
}

impl<T> BorrowBytes for &T
where
    T: BorrowBytes + ?Sized,
{
    type Output<'a>
        = T::Output<'a>
    where
        Self: 'a;

    type Error = T::Error;

    #[inline]
    fn borrow_bytes(&self, count: usize) -> Result<Self::Output<'_>, Self::Error> {
        (**self).borrow_bytes(count)
    }
}

impl<T> BorrowBytes for &mut T
where
    T: BorrowBytes + ?Sized,
{
    type Output<'a>
        = T::Output<'a>
    where
        Self: 'a;

    type Error = T::Error;

    #[inline]
    fn borrow_bytes(&self, count: usize) -> Result<Self::Output<'_>, Self::Error> {
        (**self).borrow_bytes(count)
    }
}
