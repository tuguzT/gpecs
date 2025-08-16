use core::{alloc::Layout, mem::MaybeUninit};

pub use self::{init::AlignedInitBytes, slice::AlignedUninitByteSlice};

#[cfg(feature = "alloc")]
pub use self::boxed::{AlignedUninitBoxedByteSlice, AllocError};

#[cfg(feature = "alloc")]
mod boxed;

mod init;
mod slice;

/// [Slice](prim@slice) of dynamically aligned, potentially uninitialized bytes.
pub unsafe trait AlignedBytes {
    /// Pointer to the start of [slice](AlignedBytes::as_uninit_bytes) of self.
    fn as_ptr(&self) -> *const u8;

    /// Mutable pointer to the start of [slice](AlignedBytes::as_uninit_bytes_mut) of self.
    fn as_mut_ptr(&mut self) -> *mut u8;

    /// Layout of [slice](AlignedBytes::as_uninit_bytes) of self: its length and alignment.
    fn layout(&self) -> Layout;

    /// Retrieve an uninitialized byte [prim@slice] of self,
    /// even if such bytes could be initialized.
    fn as_uninit_bytes(&self) -> &[MaybeUninit<u8>] {
        let data = self.as_ptr().cast();
        let len = self.layout().size();
        unsafe { core::slice::from_raw_parts(data, len) }
    }

    /// Retrieve a mutable uninitialized byte [prim@slice] of self,
    /// even if such bytes could be initialized.
    fn as_uninit_bytes_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        let data = self.as_mut_ptr().cast();
        let len = self.layout().size();
        unsafe { core::slice::from_raw_parts_mut(data, len) }
    }
}

unsafe impl<T> AlignedBytes for &mut T
where
    T: AlignedBytes + ?Sized,
{
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        (**self).as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        (**self).as_mut_ptr()
    }

    #[inline]
    fn layout(&self) -> Layout {
        (**self).layout()
    }
}

#[cfg(feature = "alloc")]
unsafe impl AlignedBytes for AlignedUninitBoxedByteSlice {
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        Self::as_ptr(self)
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        Self::as_mut_ptr(self)
    }

    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(self)
    }
}

unsafe impl<T> AlignedBytes for AlignedUninitByteSlice<T>
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
        Self::layout(self)
    }
}

unsafe impl<B> AlignedBytes for AlignedInitBytes<B>
where
    B: AlignedBytes + ?Sized,
{
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        Self::as_ptr(self)
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        Self::as_mut_ptr(self)
    }

    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(self)
    }
}

/// # Safety
///
/// - [`set_layout`](AlignedBytesFromLayout::set_layout) method should preserve old data
///   by copying it from the old byte slice to the new one
pub unsafe trait AlignedBytesFromLayout: AlignedBytes + Sized {
    type Error;

    fn from_layout(layout: Layout) -> Result<Self, Self::Error>;

    fn set_layout(&mut self, layout: Layout) -> Result<(), Self::Error> {
        let mut new = Self::from_layout(layout)?;

        let src = self.as_uninit_bytes();
        let dst = new.as_uninit_bytes_mut();
        let len = Ord::min(src.len(), dst.len());
        dst[..len].copy_from_slice(&src[..len]);

        *self = new;
        Ok(())
    }
}

#[cfg(feature = "alloc")]
unsafe impl AlignedBytesFromLayout for AlignedUninitBoxedByteSlice {
    type Error = AllocError;

    #[inline]
    fn from_layout(layout: Layout) -> Result<Self, Self::Error> {
        Self::new(layout)
    }

    #[inline]
    fn set_layout(&mut self, layout: Layout) -> Result<(), Self::Error> {
        Self::set_layout(self, layout)
    }
}

unsafe impl<B> AlignedBytesFromLayout for AlignedInitBytes<B>
where
    B: AlignedBytesFromLayout,
{
    type Error = B::Error;

    #[inline]
    fn from_layout(layout: Layout) -> Result<Self, Self::Error> {
        Self::from_layout(layout)
    }

    #[inline]
    fn set_layout(&mut self, layout: Layout) -> Result<(), Self::Error> {
        Self::set_layout(self, layout)
    }
}
