use itertools::{EitherOrBoth::Both, Itertools};

use crate::{
    erased::error::ErasedSoaIntoValueErrorKind,
    error::{LenMismatchError, check_layout},
    soa::field::{FieldDescriptor, IntoCopiedFieldDescriptors},
};

#[inline]
pub fn debug_assert_eq_descriptors<I, J>(a: I, b: J)
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    J: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    #[cfg(debug_assertions)]
    itertools::assert_equal(
        a.copied_field_descriptors().map(FieldDescriptor::layout),
        b.copied_field_descriptors().map(FieldDescriptor::layout),
    );

    #[cfg(not(debug_assertions))]
    let _ = (a, b);
}

pub fn check_into_value<I, J>(actual: I, expected: J) -> Result<(), ErasedSoaIntoValueErrorKind>
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    J: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    let mut actual = actual.copied_field_descriptors();
    let expected = expected.copied_field_descriptors();
    for (field_index, item) in actual.by_ref().zip_longest(expected).enumerate() {
        let Both(actual, expected) = item else {
            let descriptors_count = field_index + actual.count();
            let error = LenMismatchError::new(descriptors_count, field_index);
            return Err(error.into());
        };
        check_layout(actual.layout(), expected.layout())?;
    }
    Ok(())
}
