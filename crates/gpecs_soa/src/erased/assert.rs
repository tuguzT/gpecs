use core::{alloc::Layout, borrow::Borrow};

#[inline]
pub fn validate_layout<Fields, I>(item: I) -> Layout
where
    I: Borrow<Layout>,
{
    let layout: &Layout = item.borrow();

    let input_align = layout.align();
    let max_align = align_of::<Fields>();
    assert!(
        input_align <= max_align,
        "input alignment must be less than or equal to {max_align}, but got {input_align}",
    );
    layout.clone()
}
