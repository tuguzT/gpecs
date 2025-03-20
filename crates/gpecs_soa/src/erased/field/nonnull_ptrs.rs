use core::{
    alloc::Layout,
    ptr::{self, NonNull},
};

use super::assert::{assert_buffer_align, assert_layout, assert_value_buffer_len};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldNonNullPtr {
    layout: Layout,
    buffer: NonNull<[u8]>,
}

impl ErasedFieldNonNullPtr {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: NonNull<[u8]>) -> Self {
        assert_value_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.as_ptr().cast(), layout.align());

        Self { layout, buffer }
    }

    #[inline]
    pub fn from<T>(ptr: NonNull<T>) -> Self {
        let layout = Layout::new::<T>();
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast(), layout.size());
        let buffer = NonNull::new(ptr).expect("input pointer should be nonnull");
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> NonNull<T> {
        let Self { layout, buffer } = self;
        assert_layout::<T>(layout);

        buffer.cast()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn buffer(&self) -> NonNull<[u8]> {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> NonNull<u8> {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, NonNull<[u8]>) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}
