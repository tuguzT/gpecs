use core::{alloc::Layout, ptr};

use super::{
    super::assert::assert_layouts,
    assert::{assert_buffer_align, assert_layout, assert_value_buffer_len},
    ErasedFieldMutPtr,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldPtr {
    layout: Layout,
    buffer: *const [u8],
}

impl ErasedFieldPtr {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: *const [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.cast(), layout.align());

        Self { layout, buffer }
    }

    #[inline]
    pub fn dangling(layout: Layout) -> Self {
        let data = ptr::without_provenance(layout.align());
        let buffer = ptr::slice_from_raw_parts(data, layout.size());
        Self::new(layout, buffer)
    }

    #[inline]
    pub fn from<T>(ptr: *const T) -> Self {
        let layout = Layout::new::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), layout.size());
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> *const T {
        let Self { layout, buffer } = self;
        assert_layout::<T>(&layout);

        buffer.cast()
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldMutPtr {
        let Self { layout, buffer } = self;
        ErasedFieldMutPtr::new(layout, buffer.cast_mut())
    }

    #[inline]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { layout, buffer } = self;

        let data = unsafe { buffer.cast::<u8>().add(count * layout.size()) };
        let len = layout.size();
        let buffer = ptr::slice_from_raw_parts(data, len);
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(&self, origin: &Self) -> isize {
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
    pub fn buffer(&self) -> *const [u8] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_ptr(self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, *const [u8]) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}
