use core::{alloc::Layout, any::type_name};

#[cold]
#[track_caller]
#[inline(never)]
fn assert_value_buffer_len_failed(buffer_len: usize, layout_size: usize) -> ! {
    panic!("buffer len {buffer_len} should match layout size {layout_size}")
}

#[inline]
#[track_caller]
pub fn assert_value_buffer_len(buffer_len: usize, layout_size: usize) {
    if buffer_len == layout_size {
        return;
    }
    assert_value_buffer_len_failed(buffer_len, layout_size)
}

#[cold]
#[track_caller]
#[inline(never)]
fn assert_slice_buffer_len_failed(buffer_len: usize, layout_size: usize, len: usize) -> ! {
    panic!("buffer len {buffer_len} divided by layout size {layout_size} should be equal to {len}")
}

#[inline]
#[track_caller]
pub fn assert_slice_buffer_len(buffer_len: usize, layout_size: usize, len: usize) {
    if layout_size == 0 && buffer_len == 0 {
        return;
    }
    if buffer_len.div_ceil(layout_size) == len {
        return;
    }
    assert_slice_buffer_len_failed(buffer_len, layout_size, len)
}

#[cold]
#[track_caller]
#[inline(never)]
fn assert_buffer_align_failed(layout_align: usize) -> ! {
    panic!("buffer should be aligned to {layout_align}")
}

#[inline]
#[track_caller]
pub fn assert_buffer_align(buffer: *const u8, layout_align: usize) {
    if buffer.align_offset(layout_align) == 0 {
        return;
    }
    assert_buffer_align_failed(layout_align)
}

#[cold]
#[track_caller]
#[inline(never)]
fn assert_layout_failed<T>(layout: Layout) -> ! {
    let target_layout = Layout::new::<T>();
    let type_name = type_name::<T>();
    panic!("layout {target_layout:?} of type {type_name} should match layout {layout:?}")
}

#[inline]
#[track_caller]
pub fn assert_layout<T>(layout: Layout) {
    if layout == Layout::new::<T>() {
        return;
    }
    assert_layout_failed::<T>(layout)
}
