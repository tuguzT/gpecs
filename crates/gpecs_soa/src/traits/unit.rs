use core::{
    alloc::{Layout, LayoutError},
    array,
    ptr::{self, NonNull},
};

use crate::traits::{FieldLayouts, SoaAllocContext, SoaAllocTrusted};

impl<'a> FieldLayouts<'a> for () {
    type Output = [Layout; 0];
    type OutputIter = array::IntoIter<Layout, 0>;
    type OutputItem = Layout;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        []
    }
}

unsafe impl SoaAllocContext<()> for () {
    #[inline]
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        Layout::array::<()>(capacity)
    }

    #[inline]
    fn buffer_align(&self) -> usize {
        align_of::<()>()
    }

    #[inline]
    fn packed_size_of_fields(&self) -> Option<usize> {
        Some(size_of::<()>())
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
    unsafe fn mut_ptrs_from_buffer(&self, buffer: *mut u8, _capacity: usize) -> Self::MutPtrs<'_> {
        buffer.cast()
    }

    #[inline]
    unsafe fn nonnull_ptrs_from_buffer(
        &self,
        buffer: NonNull<u8>,
        _capacity: usize,
    ) -> Self::NonNullPtrs<'_> {
        buffer.cast()
    }

    #[inline]
    unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
        unsafe { ptr::copy(src, dst, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_copy_forward(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from(src, count) }
    }

    #[inline]
    unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
        unsafe { ptr::copy(src, dst, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_copy_backward(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from(src, count) }
    }
}

unsafe impl SoaAllocTrusted for () {}
