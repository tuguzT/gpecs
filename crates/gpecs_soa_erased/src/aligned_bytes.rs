use alloc::alloc::{alloc, dealloc, handle_alloc_error, realloc};
use core::{
    alloc::Layout,
    ptr::{self, NonNull},
};

#[derive(Debug)]
pub struct AlignedBytes {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl AlignedBytes {
    #[inline]
    pub fn new(layout: Layout) -> Self {
        let ptr = match layout.size() {
            0 => ptr::without_provenance_mut(layout.align()),
            _ => unsafe { alloc(layout) },
        };
        let Some(ptr) = NonNull::new(ptr) else {
            handle_alloc_error(layout);
        };
        Self { ptr, layout }
    }

    #[inline]
    #[allow(dead_code)]
    pub unsafe fn from_parts(ptr: *mut u8, layout: Layout) -> Self {
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Self { ptr, layout }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn into_parts(self) -> (*mut u8, Layout) {
        let Self { ptr, layout } = self;
        (ptr.as_ptr(), layout)
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr.as_ptr()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn set_layout(&mut self, layout: Layout) {
        let new_layout = layout;
        let Self { ptr, layout } = *self;
        if layout == new_layout {
            return;
        }

        if new_layout.size() != 0 && layout.align() == new_layout.align() {
            let new_ptr = unsafe { realloc(ptr.as_ptr(), layout, new_layout.size()) };
            let Some(new_ptr) = NonNull::new(new_ptr) else {
                handle_alloc_error(new_layout);
            };

            self.ptr = new_ptr;
            self.layout = new_layout;
            return;
        }

        let mut new = Self::new(new_layout);
        unsafe {
            let count = Ord::min(layout.size(), new_layout.size());
            ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_mut_ptr(), count);
        }
        *self = new;
    }
}

impl AsRef<AlignedBytes> for AlignedBytes {
    #[inline]
    fn as_ref(&self) -> &AlignedBytes {
        self
    }
}

impl AsMut<AlignedBytes> for AlignedBytes {
    #[inline]
    fn as_mut(&mut self) -> &mut AlignedBytes {
        self
    }
}

impl Drop for AlignedBytes {
    fn drop(&mut self) {
        let Self { ptr, layout } = *self;
        if layout.size() == 0 {
            return;
        }
        unsafe { dealloc(ptr.as_ptr(), layout) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let layout = Layout::new::<u64>();
        let mut bytes = AlignedBytes::new(layout);

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
        let mut bytes = AlignedBytes::new(layout);

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
        let mut bytes = AlignedBytes::new(layout);
        unsafe { bytes.as_mut_ptr().cast::<u64>().write(42) }

        let layout = Layout::new::<u128>();
        bytes.set_layout(layout);
        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);

        let ptr = bytes.as_mut_ptr().cast::<u64>();
        let value = unsafe { ptr.read() };
        assert_eq!(value, 42);

        let layout = Layout::new::<u32>();
        bytes.set_layout(layout);
        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);

        let ptr = bytes.as_mut_ptr().cast::<u32>();
        let value = unsafe { ptr.read() };
        assert_eq!(value, 42);

        let layout = Layout::new::<(u32, u32)>();
        bytes.set_layout(layout);
        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);

        let ptr = bytes.as_mut_ptr().cast::<u32>();
        let value = unsafe { ptr.read() };
        assert_eq!(value, 42);

        let layout = Layout::new::<()>();
        bytes.set_layout(layout);
        assert_eq!(bytes.layout(), layout);
        assert_eq!(bytes.as_ptr().align_offset(layout.align()), 0);

        let ptr = bytes.as_mut_ptr().cast::<()>();
        let value = unsafe { ptr.read() };
        assert_eq!(value, ());
    }
}
