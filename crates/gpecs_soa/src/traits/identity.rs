use core::{
    alloc::{Layout, LayoutError},
    array, ptr,
};

use crate::{
    field::FieldLayouts,
    identity::Identity,
    traits::{AllocSoaContext, AllocSoaTrusted},
};

impl<'a, T> FieldLayouts<'a, Identity<T>> for () {
    type Output = [Layout; 1];
    type OutputIter = array::IntoIter<Layout, 1>;
    type OutputItem = Layout;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        [Layout::new::<T>()]
    }
}

unsafe impl<T> AllocSoaContext<Identity<T>> for () {
    #[inline]
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        Layout::array::<T>(capacity)
    }

    #[inline]
    fn capacity_from(&self, buffer_layout: Layout) -> usize {
        buffer_layout
            .size()
            .checked_div(size_of::<T>())
            .unwrap_or(usize::MAX)
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

unsafe impl<T> AllocSoaTrusted for Identity<T> {}
