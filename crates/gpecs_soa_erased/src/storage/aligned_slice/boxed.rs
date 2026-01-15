use alloc::alloc::{alloc, dealloc, realloc};
use core::{
    alloc::Layout,
    error::Error,
    fmt::{self, Display},
    mem::MaybeUninit,
    ptr::{self, NonNull},
    slice,
};

use crate::storage::{AlignedSlice, AlignedSliceFromLayout};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
/// Just a copy of unstable [`core::alloc::AllocError`].
pub struct AllocError;

impl Display for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("memory allocation failed")
    }
}

impl Error for AllocError {}

#[derive(Debug)]
pub struct AlignedUninitBoxedByteSlice {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl AlignedUninitBoxedByteSlice {
    #[inline]
    pub fn new(layout: Layout) -> Result<Self, AllocError> {
        let ptr = match layout.size() {
            0 => ptr::without_provenance_mut(layout.align()),
            _ => unsafe { alloc(layout) },
        };
        let Some(ptr) = NonNull::new(ptr) else {
            return Err(AllocError);
        };

        let me = Self { ptr, layout };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(ptr: NonNull<u8>, layout: Layout) -> Self {
        Self { ptr, layout }
    }

    #[inline]
    pub fn into_parts(self) -> (NonNull<u8>, Layout) {
        let Self { ptr, layout } = self;
        (ptr, layout)
    }

    #[inline]
    pub fn as_nonnull_ptr(&self) -> NonNull<u8> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr.as_ptr().cast_const()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn as_uninit_slice(&self) -> &[MaybeUninit<u8>] {
        let Self { ptr, layout } = *self;

        let data = ptr.as_ptr().cast_const().cast();
        let len = layout.size();
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    pub fn as_mut_uninit_slice(&mut self) -> &mut [MaybeUninit<u8>] {
        let Self { ptr, layout } = *self;

        let data = ptr.as_ptr().cast();
        let len = layout.size();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    pub fn set_layout(&mut self, layout: Layout) -> Result<(), AllocError> {
        let new_layout = layout;
        let Self { ptr, layout } = *self;
        if layout == new_layout {
            return Ok(());
        }

        if new_layout.size() != 0 && layout.align() == new_layout.align() {
            let new_ptr = unsafe { realloc(ptr.as_ptr(), layout, new_layout.size()) };
            let Some(new_ptr) = NonNull::new(new_ptr) else {
                return Err(AllocError);
            };

            self.ptr = new_ptr;
            self.layout = new_layout;
            return Ok(());
        }

        let mut new = Self::new(new_layout)?;
        unsafe {
            let count = Ord::min(layout.size(), new_layout.size());
            ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_mut_ptr(), count);
        }
        *self = new;
        Ok(())
    }
}

impl AsRef<Self> for AlignedUninitBoxedByteSlice {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Self> for AlignedUninitBoxedByteSlice {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl AsRef<[MaybeUninit<u8>]> for AlignedUninitBoxedByteSlice {
    #[inline]
    fn as_ref(&self) -> &[MaybeUninit<u8>] {
        self.as_uninit_slice()
    }
}

impl AsMut<[MaybeUninit<u8>]> for AlignedUninitBoxedByteSlice {
    #[inline]
    fn as_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.as_mut_uninit_slice()
    }
}

impl Drop for AlignedUninitBoxedByteSlice {
    fn drop(&mut self) {
        let Self { ptr, layout } = *self;
        if layout.size() == 0 {
            return;
        }
        unsafe { dealloc(ptr.as_ptr(), layout) }
    }
}

unsafe impl AlignedSlice<u8> for AlignedUninitBoxedByteSlice {
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

unsafe impl AlignedSliceFromLayout<u8> for AlignedUninitBoxedByteSlice {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let layout = Layout::new::<u64>();
        let mut bytes = AlignedUninitBoxedByteSlice::new(layout).unwrap();

        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);

        let ptr = bytes.as_mut_ptr().cast::<u64>();
        let value = unsafe {
            ptr.write(42);
            ptr.read()
        };
        assert_eq!(value, 42);
    }

    #[test]
    fn new_zst() {
        let layout = Layout::new::<()>();
        let mut bytes = AlignedUninitBoxedByteSlice::new(layout).unwrap();

        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);
        assert_eq!(
            bytes.as_mut_ptr(),
            ptr::without_provenance_mut(layout.align()),
        );

        let ptr = bytes.as_mut_ptr().cast::<()>();
        let value = unsafe {
            ptr.write(());
            ptr.read()
        };
        assert_eq!(value, ());
    }

    #[test]
    fn set_layout() {
        let layout = Layout::new::<u64>();
        let mut bytes = AlignedUninitBoxedByteSlice::new(layout).unwrap();
        unsafe { bytes.as_mut_ptr().cast::<u64>().write(42) }

        let layout = Layout::new::<u128>();
        bytes.set_layout(layout).unwrap();
        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);

        let ptr = bytes.as_mut_ptr().cast::<u64>();
        let value = unsafe { ptr.read() };
        assert_eq!(value, 42);

        let layout = Layout::new::<u32>();
        bytes.set_layout(layout).unwrap();
        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);

        let ptr = bytes.as_mut_ptr().cast::<u32>();
        let value = unsafe { ptr.read() };
        assert_eq!(value, 42);

        let layout = Layout::new::<(u32, u32)>();
        bytes.set_layout(layout).unwrap();
        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);

        let ptr = bytes.as_mut_ptr().cast::<u32>();
        let value = unsafe { ptr.read() };
        assert_eq!(value, 42);

        let layout = Layout::new::<()>();
        bytes.set_layout(layout).unwrap();
        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);

        let ptr = bytes.as_mut_ptr().cast::<()>();
        let value = unsafe { ptr.read() };
        assert_eq!(value, ());
    }
}
