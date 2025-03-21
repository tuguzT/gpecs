use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use super::{
    assert::{assert_buffer_align, assert_layout, assert_value_buffer_len},
    ErasedFieldPtr,
};

#[derive(Clone, Copy)]
pub struct ErasedFieldRef<'a> {
    layout: Layout,
    buffer: &'a [u8],
    no_send_sync: PhantomData<*const u8>,
}

impl<'a> ErasedFieldRef<'a> {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: &'a [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.as_ptr(), layout.align());

        Self {
            layout,
            buffer,
            no_send_sync: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(r#ref: &'a T) -> Self {
        let layout = Layout::new::<T>();
        let data = ptr::from_ref(r#ref).cast();
        let buffer = unsafe { slice::from_raw_parts(data, layout.size()) };
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self) -> &'a T {
        let Self { layout, buffer, .. } = self;
        assert_layout::<T>(layout);

        let ptr = buffer.as_ptr().cast();
        unsafe { &*ptr }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> &T {
        let Self { layout, buffer, .. } = self;
        assert_layout::<T>(layout);

        let ptr = buffer.as_ptr().cast();
        unsafe { &*ptr }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.as_ptr()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { layout, buffer, .. } = *self;
        let buffer = ptr::from_ref(buffer);
        ErasedFieldPtr::new(layout, buffer)
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, &'a [u8]) {
        let Self { layout, buffer, .. } = self;
        (layout, buffer)
    }
}

impl Debug for ErasedFieldRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { layout, buffer, .. } = self;
        f.debug_struct("ErasedFieldRef")
            .field("layout", layout)
            .field("buffer", buffer)
            .finish()
    }
}
