use crate::soa::field::FieldDescriptor;

#[cfg(debug_assertions)]
use crate::soa::field::CopiedFieldDescriptors;

#[inline]
pub fn debug_assert_eq_descriptors<I, J>(a: I, b: J)
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    J: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    #[cfg(debug_assertions)]
    itertools::assert_equal(
        CopiedFieldDescriptors(a.into_iter()).map(FieldDescriptor::layout),
        CopiedFieldDescriptors(b.into_iter()).map(FieldDescriptor::layout),
    );

    #[cfg(not(debug_assertions))]
    let _ = (a, b);
}
