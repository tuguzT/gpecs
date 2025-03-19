use core::{alloc::Layout, ptr};

use super::assert::{assert_buffer_align, assert_layout, assert_value_buffer_len};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldMutPtr {
    layout: Layout,
    buffer: *mut [u8],
}

impl ErasedFieldMutPtr {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: *mut [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.cast(), layout.align());

        Self { layout, buffer }
    }

    #[inline]
    pub fn from<T>(ptr: *mut T) -> Self {
        let layout = Layout::new::<T>();
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), layout.size());
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> *mut T {
        let Self { layout, buffer } = self;
        assert_layout::<T>(&layout);

        buffer.cast()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn buffer(&self) -> *mut [u8] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, *mut [u8]) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}
