use core::{alloc::Layout, mem::MaybeUninit, slice};

use crate::storage::AddressableUnit;

/// [Slice](prim@slice) of dynamically aligned, potentially uninitialized addressible units.
pub unsafe trait AlignedSlice<A>
where
    A: AddressableUnit,
{
    /// Pointer to the start of [slice](AlignedSlice::as_uninit_slice) of self.
    fn as_ptr(&self) -> *const A;

    /// Mutable pointer to the start of [slice](AlignedSlice::as_mut_uninit_slice) of self.
    fn as_mut_ptr(&mut self) -> *mut A;

    /// Layout of [slice](AlignedSlice::as_uninit_slice) of self: its length and alignment.
    fn layout(&self) -> Layout;

    /// Retrieve an uninitialized [slice](prim@slice) of self,
    /// even if such addressible units could be initialized.
    fn as_uninit_slice(&self) -> &[MaybeUninit<A>] {
        let data = self.as_ptr().cast();
        let len = self.layout().size();
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Retrieve a mutable uninitialized [slice](prim@slice) of self,
    /// even if such addressible units could be initialized.
    fn as_mut_uninit_slice(&mut self) -> &mut [MaybeUninit<A>] {
        let data = self.as_mut_ptr().cast();
        let len = self.layout().size();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }
}

unsafe impl<A, T> AlignedSlice<A> for &mut T
where
    A: AddressableUnit,
    T: AlignedSlice<A> + ?Sized,
{
    #[inline]
    fn as_ptr(&self) -> *const A {
        (**self).as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut A {
        (**self).as_mut_ptr()
    }

    #[inline]
    fn layout(&self) -> Layout {
        (**self).layout()
    }

    #[inline]
    fn as_uninit_slice(&self) -> &[MaybeUninit<A>] {
        (**self).as_uninit_slice()
    }

    #[inline]
    fn as_mut_uninit_slice(&mut self) -> &mut [MaybeUninit<A>] {
        (**self).as_mut_uninit_slice()
    }
}

/// An extension of [aligned slice](AlignedSlice) type
/// which could potentially be constructed from a given layout.
///
/// # Safety
///
/// - [`set_layout()`](AlignedSliceFromLayout::set_layout()) should preserve old data
///   by copying it from the old byte slice to the new one.
pub unsafe trait AlignedSliceFromLayout<A>: AlignedSlice<A> + Sized
where
    A: AddressableUnit,
{
    /// An error type which could occur during construction of self from a given layout.
    type Error;

    /// Construct self from the given layout.
    ///
    /// # Errors
    ///
    /// This function returns an error if the given layout is invalid for this type.
    fn from_layout(layout: Layout) -> Result<Self, Self::Error>;

    /// Change the layout of self to the given one.
    ///
    /// # Errors
    ///
    /// This function returns an error if the given layout is invalid for this type.
    fn set_layout(&mut self, layout: Layout) -> Result<(), Self::Error> {
        let mut new = Self::from_layout(layout)?;

        let src = self.as_uninit_slice();
        let dst = new.as_mut_uninit_slice();
        let len = usize::min(src.len(), dst.len());
        dst[..len].copy_from_slice(&src[..len]);

        *self = new;
        Ok(())
    }
}
