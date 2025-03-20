use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use super::assert::{assert_buffer_align, assert_layout, assert_value_buffer_len};

#[derive(PartialEq, Eq, Hash)]
pub struct ErasedFieldRefMut<'a> {
    layout: Layout,
    buffer: &'a mut [u8],
    no_send_sync: PhantomData<*const u8>,
}

impl<'a> ErasedFieldRefMut<'a> {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: &'a mut [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.as_ptr(), layout.align());

        Self {
            layout,
            buffer,
            no_send_sync: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(r#ref: &'a mut T) -> Self {
        let layout = Layout::new::<T>();
        let data = ptr::from_mut(r#ref).cast();
        let buffer = unsafe { slice::from_raw_parts_mut(data, layout.size()) };
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self) -> &'a mut T {
        let Self { layout, buffer, .. } = self;
        assert_layout::<T>(layout);

        let ptr = buffer.as_mut_ptr().cast();
        unsafe { &mut *ptr }
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
    #[track_caller]
    pub unsafe fn cast_mut<T>(&mut self) -> &mut T {
        let Self { layout, buffer, .. } = self;
        assert_layout::<T>(layout);

        let ptr = buffer.as_mut_ptr().cast();
        unsafe { &mut *ptr }
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
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.as_mut_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, &'a mut [u8]) {
        let Self { layout, buffer, .. } = self;
        (layout, buffer)
    }
}

impl Debug for ErasedFieldRefMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { layout, buffer, .. } = self;
        f.debug_struct("ErasedFieldRefMut")
            .field("layout", layout)
            .field("buffer", buffer)
            .finish()
    }
}
