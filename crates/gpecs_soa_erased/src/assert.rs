use itertools::{EitherOrBoth::Both, Itertools};

use crate::{
    error::DowncastErrorKind,
    error::{LenMismatchError, check_layout},
    soa::{field::buffer_offsets, layout::WithLayout},
};

#[inline]
pub fn assert_layouts<I, J>(a: I, b: J) -> usize
where
    I: IntoIterator<Item: WithLayout>,
    J: IntoIterator<Item: WithLayout>,
{
    #[cfg(debug_assertions)]
    {
        let mut len = 0;
        let a = a.into_iter().inspect(|_| len += 1);

        itertools::assert_equal(
            a.into_iter().map(|item| item.layout()),
            b.into_iter().map(|item| item.layout()),
        );
        len
    }

    #[cfg(not(debug_assertions))]
    {
        let len = a.into_iter().count();
        assert!(
            len == b.into_iter().count(),
            "layouts should have the same length"
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
    I: IntoIterator<Item: WithLayout>,
    J: IntoIterator<Item: WithLayout>,
{
    let mut actual = buffer_offsets(actual, capacity);
    let mut expected = buffer_offsets(expected, capacity);
    for (field_index, item) in actual.by_ref().zip_longest(expected.by_ref()).enumerate() {
        let Both(actual, expected) = item else {
            let layouts_count = field_index + actual.count();
            let error = unsafe { LenMismatchError::new_unchecked(layouts_count, field_index) };
            return Err(error.into());
        };

        let actual = actual?.desc.layout();
        let expected = expected?.desc.layout();
        check_layout(actual, expected)?;
    }
    check_layout(actual.into_buffer_layout(), expected.into_buffer_layout())?;
    Ok(())
}
