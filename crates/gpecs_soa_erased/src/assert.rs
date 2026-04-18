use itertools::{EitherOrBoth::Both, Itertools};

use crate::{
    error::DowncastErrorKind,
    error::{LenMismatchError, check_layout},
    soa::field::{FieldDescriptor, buffer_offsets},
};

#[inline]
pub fn assert_descriptors<I, J>(a: I, b: J) -> usize
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    J: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    #[cfg(debug_assertions)]
    {
        use crate::soa::field::IntoCopiedFieldDescriptors;

        let mut len = 0;
        let a = a.into_iter().inspect(|_| len += 1);

        itertools::assert_equal(
            a.copied_field_descriptors().map(FieldDescriptor::layout),
            b.copied_field_descriptors().map(FieldDescriptor::layout),
        );
        len
    }

    #[cfg(not(debug_assertions))]
    {
        let len = a.into_iter().count();
        assert!(
            len == b.into_iter().count(),
            "descriptors should have the same length"
        );
        len
    }
}

pub fn check_downcast<I, J>(
    actual: I,
    expected: J,
    capacity: usize,
) -> Result<(), DowncastErrorKind>
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
    J: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    let mut actual = buffer_offsets(actual, capacity);
    let mut expected = buffer_offsets(expected, capacity);
    for (field_index, item) in actual.by_ref().zip_longest(expected.by_ref()).enumerate() {
        let Both(actual, expected) = item else {
            let descriptors_count = field_index + actual.count();
            let error = unsafe { LenMismatchError::new_unchecked(descriptors_count, field_index) };
            return Err(error.into());
        };

        let actual = actual?;
        let expected = expected?;
        check_layout(actual.desc.layout(), expected.desc.layout())?;
    }
    check_layout(actual.into_layout(), expected.into_layout())?;
    Ok(())
}
