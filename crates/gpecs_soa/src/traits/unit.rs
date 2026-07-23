use core::{
    alloc::{Layout, LayoutError},
    array, ptr,
};

use crate::traits::{AllocSoaContext, AllocSoaTrusted, FieldLayouts};

impl<'a> FieldLayouts<'a> for () {
    type Output = [Layout; 0];
    type OutputIter = array::IntoIter<Layout, 0>;
    type OutputItem = Layout;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        []
    }
}

unsafe impl AllocSoaContext<()> for () {
    #[inline]
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        Layout::array::<()>(capacity)
    }

    #[inline]
    fn capacity_from(&self, _buffer_layout: Layout) -> usize {
        usize::MAX
    }

    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, _capacity: usize) -> Self::Ptrs<'_> {
        buffer.cast()
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, _capacity: usize) -> Self::MutPtrs<'_> {
        buffer.cast()
    }

    #[inline]
    unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
        unsafe { ptr::copy(src, dst, count) }
    }

    #[inline]
    unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
        unsafe { ptr::copy(src, dst, count) }
    }
}

unsafe impl AllocSoaTrusted for () {}
