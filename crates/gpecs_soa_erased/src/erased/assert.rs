use core::ptr;

use crate::soa::field::FieldDescriptor;

#[inline]
pub fn assert_descriptors(a: &[FieldDescriptor], b: &[FieldDescriptor]) {
    if ptr::eq(a, b) {
        return;
    }
    itertools::assert_equal(
        a.iter().map(FieldDescriptor::layout),
        b.iter().map(FieldDescriptor::layout),
    );
}
