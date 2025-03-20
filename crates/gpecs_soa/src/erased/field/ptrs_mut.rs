use core::{alloc::Layout, ptr};

use super::{
    super::assert::assert_layouts,
    assert::{assert_buffer_align, assert_layout, assert_value_buffer_len},
    ErasedFieldPtr,
};

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
    pub fn dangling(layout: Layout) -> Self {
        let data = ptr::without_provenance_mut(layout.align());
        let buffer = ptr::slice_from_raw_parts_mut(data, layout.size());
        Self::new(layout, buffer)
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
    pub fn cast_const(self) -> ErasedFieldPtr {
        let Self { layout, buffer } = self;
        ErasedFieldPtr::new(layout, buffer.cast_const())
    }

    #[inline]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { layout, buffer } = self;

        let data = unsafe { buffer.cast::<u8>().add(count * layout.size()) };
        let len = layout.size();
        let buffer = ptr::slice_from_raw_parts_mut(data, len);
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(&self, origin: &ErasedFieldPtr) -> isize {
        let Self { layout, .. } = self;
        assert_layouts(layout, origin.layout());

        let offset = unsafe { self.as_ptr().offset_from(origin.as_ptr()) };
        let field_size = layout
            .size()
            .try_into()
            .expect("layout size should not exceed `isize::MAX`");
        offset
            .checked_div(field_size)
            .expect("erased field pointer should not be a ZST")
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
    pub fn into_ptr(self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, *mut [u8]) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}
