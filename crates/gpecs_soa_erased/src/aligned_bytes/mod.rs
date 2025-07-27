use core::{alloc::Layout, mem::MaybeUninit, ptr};

use crate::error::{LenMismatchError, check_len};

pub use self::{
    boxed::{AlignedBoxedByteSlice, AllocError},
    slice::AlignedByteSlice,
};

mod boxed;
mod slice;

/// [Slice](prim@slice) of dynamically-aligned uninitialized bytes.
pub unsafe trait AlignedBytes {
    /// Pointer to the start of aligned uninitialized byte [prim@slice].
    fn as_ptr(&self) -> *const u8;

    /// Mutable pointer to the start of aligned uninitialized byte [prim@slice].
    fn as_mut_ptr(&mut self) -> *mut u8;

    /// Layout of aligned uninitialized byte [prim@slice]: its length and alignment.
    fn layout(&self) -> Layout;

    fn copy_from(&mut self, slice: &[u8]) -> Result<(), LenMismatchError> {
        let expected = self.layout().size();
        let len = slice.len();
        check_len(len, expected)?;

        let src = slice.as_ptr();
        let dst = self.as_mut_ptr();
        unsafe {
            ptr::copy_nonoverlapping(src, dst, len);
        }
        Ok(())
    }
}

unsafe impl AlignedBytes for AlignedBoxedByteSlice {
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        AlignedBoxedByteSlice::as_ptr(self)
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        AlignedBoxedByteSlice::as_mut_ptr(self)
    }

    #[inline]
    fn layout(&self) -> Layout {
        AlignedBoxedByteSlice::layout(self)
    }
}

unsafe impl<T> AlignedBytes for AlignedByteSlice<T>
where
    T: AsRef<[MaybeUninit<u8>]> + AsMut<[MaybeUninit<u8>]> + ?Sized,
{
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        let slice = self.as_slice();
        slice.as_ptr().cast()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        let slice = self.as_mut_slice();
        slice.as_mut_ptr().cast()
    }

    #[inline]
    fn layout(&self) -> Layout {
        AlignedByteSlice::layout(self)
    }
}

pub unsafe trait AlignedBytesFromLayout: AlignedBytes + Sized {
    type Error;

    fn from_layout(layout: Layout) -> Result<Self, Self::Error>;

    fn set_layout(&mut self, layout: Layout) -> Result<(), Self::Error> {
        *self = Self::from_layout(layout)?;
        Ok(())
    }
}

unsafe impl AlignedBytesFromLayout for AlignedBoxedByteSlice {
    type Error = AllocError;

    #[inline]
    fn from_layout(layout: Layout) -> Result<Self, Self::Error> {
        AlignedBoxedByteSlice::new(layout)
    }

    #[inline]
    fn set_layout(&mut self, layout: Layout) -> Result<(), Self::Error> {
        AlignedBoxedByteSlice::set_layout(self, layout)
    }
}
