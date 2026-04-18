use core::{alloc::Layout, slice};

/// [Slice](prim@slice) of dynamically aligned data.
pub unsafe trait AlignedStorage {
    type Item;

    /// Pointer to the start of [slice](AlignedStorage::as_slice) of self.
    fn as_ptr(&self) -> *const Self::Item;

    /// Mutable pointer to the start of [slice](AlignedStorage::as_mut_slice) of self.
    fn as_mut_ptr(&mut self) -> *mut Self::Item;

    /// Layout of [slice](AlignedStorage::as_slice) of self: its length and alignment.
    fn layout(&self) -> Layout;

    /// Retrieve an uninitialized [slice](prim@slice) of self.
    fn as_slice(&self) -> &[Self::Item] {
        let data = self.as_ptr();
        let len = self.layout().size();
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Retrieve a mutable uninitialized [slice](prim@slice) of self.
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        let data = self.as_mut_ptr();
        let len = self.layout().size();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }
}

unsafe impl<T> AlignedStorage for &mut T
where
    T: AlignedStorage + ?Sized,
{
    type Item = T::Item;

    #[inline]
    fn as_ptr(&self) -> *const Self::Item {
        (**self).as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        (**self).as_mut_ptr()
    }

    #[inline]
    fn layout(&self) -> Layout {
        (**self).layout()
    }

    #[inline]
    fn as_slice(&self) -> &[Self::Item] {
        (**self).as_slice()
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        (**self).as_mut_slice()
    }
}

/// An extension of [aligned slice](AlignedStorage) type
/// which could potentially be constructed from a given layout.
///
/// # Safety
///
/// - [`set_layout()`](AlignedStorageFromLayout::set_layout()) should preserve old data
///   by copying it from the old slice to the new one.
pub unsafe trait AlignedStorageFromLayout: AlignedStorage + Sized {
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
    fn set_layout(&mut self, layout: Layout) -> Result<(), Self::Error>;
}
