use crate::soa::field::FieldDescriptor;

#[inline]
#[cfg(debug_assertions)]
pub fn debug_assert_eq_descriptors(a: &[FieldDescriptor], b: &[FieldDescriptor]) {
    if core::ptr::eq(a, b) {
        return;
    }

    itertools::assert_equal(
        a.iter().copied().map(FieldDescriptor::layout),
        b.iter().copied().map(FieldDescriptor::layout),
    );
}

#[inline]
#[cfg(not(debug_assertions))]
pub fn debug_assert_eq_descriptors(a: &[FieldDescriptor], b: &[FieldDescriptor]) {
    let _ = (a, b);
}
