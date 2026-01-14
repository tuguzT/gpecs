use core::ptr;

use crate::soa::field::FieldDescriptor;

#[inline]
#[cfg_attr(not(debug_assertions), expect(clippy::needless_return))]
pub fn assert_eq_descriptors(a: &[FieldDescriptor], b: &[FieldDescriptor]) {
    if ptr::eq(a, b) {
        return;
    }

    #[cfg(debug_assertions)]
    itertools::assert_equal(
        a.iter().map(FieldDescriptor::layout),
        b.iter().map(FieldDescriptor::layout),
    );
}
