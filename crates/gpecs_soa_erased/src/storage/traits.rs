use core::{alloc::Layout, fmt::UpperHex, mem::MaybeUninit, slice};

/// Marker trait for an addressible unit of memory for a given target (CPU or GPU).
pub trait AddressableUnit: UpperHex + Copy + Default + 'static {}

/// The smallest addressible unit for any CPU target.
impl AddressableUnit for u8 {}

/// The guaranteed addressible unit for any GPU target.
impl AddressableUnit for u32 {}

/// [Slice](prim@slice) of dynamically aligned, potentially uninitialized addressible units.
pub unsafe trait AlignedStorage<A>
where
    A: AddressableUnit,
{
    /// Pointer to the start of [slice](AlignedStorage::as_uninit_slice) of self.
    fn as_ptr(&self) -> *const A;

    /// Mutable pointer to the start of [slice](AlignedStorage::as_mut_uninit_slice) of self.
    fn as_mut_ptr(&mut self) -> *mut A;

    /// Layout of [slice](AlignedStorage::as_uninit_slice) of self: its length and alignment.
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

unsafe impl<A, T> AlignedStorage<A> for &mut T
where
    A: AddressableUnit,
    T: AlignedStorage<A> + ?Sized,
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

/// An extension of [aligned slice](AlignedStorage) type
/// which could potentially be constructed from a given layout.
///
/// # Safety
///
/// - [`set_layout()`](AlignedStorageFromLayout::set_layout()) should preserve old data
///   by copying it from the old byte slice to the new one.
pub unsafe trait AlignedStorageFromLayout<A>: AlignedStorage<A> + Sized
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
