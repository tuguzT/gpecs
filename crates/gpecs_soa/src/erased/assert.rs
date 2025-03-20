use core::{alloc::Layout, borrow::Borrow};

#[cold]
#[track_caller]
#[inline(never)]
fn validate_layout_failed(input_align: usize, max_align: usize) -> ! {
    panic!("input alignment {input_align} must be less than or equal to {max_align}")
}

#[inline]
#[track_caller]
pub fn validate_layout<Fields>(layout: impl Borrow<Layout>) {
    let input_align = layout.borrow().align();
    let max_align = align_of::<Fields>();
    if input_align <= max_align {
        return;
    }
    validate_layout_failed(input_align, max_align)
}

#[cold]
#[track_caller]
#[inline(never)]
fn assert_same_len_failed(base_len: usize, len: usize) -> ! {
    panic!("length {len} should be equal to {base_len}")
}

#[inline]
#[track_caller]
pub fn assert_same_len(base_len: usize, len: usize) {
    if base_len == len {
        return;
    }
    assert_same_len_failed(base_len, len)
}

#[cold]
#[track_caller]
#[inline(never)]
fn assert_layouts_failed(first: &Layout, second: &Layout) -> ! {
    panic!("layouts {first:?} and {second:?} should match")
}

#[inline]
#[track_caller]
pub fn assert_layouts(first: impl Borrow<Layout>, second: impl Borrow<Layout>) {
    let first = first.borrow();
    let second = second.borrow();
    if first == second {
        return;
    }
    assert_layouts_failed(first, second)
}
