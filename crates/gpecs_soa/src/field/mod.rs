use core::{alloc::LayoutError, num::NonZeroUsize};

use crate::layout::WithLayout;

pub use self::{
    buffer_layout::BufferLayout,
    buffer_offsets::{BufferOffset, BufferOffsets},
    field_layouts::{
        FieldLayouts, FieldLayoutsItem, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned,
    },
    into_layouts::{IntoFieldLayouts, IntoFieldLayoutsIter},
};

mod buffer_layout;
mod buffer_offsets;
mod field_layouts;
mod into_layouts;

/// Calculates offsets for each provided field repeated `capacity` times in a single buffer.
#[inline]
pub fn buffer_offsets<I>(fields: I, capacity: usize) -> BufferOffsets<I::IntoIter>
where
    I: IntoIterator<Item: WithLayout>,
{
    let state = BufferLayout::new(capacity, NonZeroUsize::MIN);
    let fields = fields.into_iter();
    unsafe { BufferOffsets::from_parts(state, fields) }
}

/// Calculates total layout needed to store each provided field repeated `capacity` times in a single buffer.
#[inline]
pub fn buffer_layout<I>(fields: I, capacity: usize) -> Result<BufferLayout, LayoutError>
where
    I: IntoIterator<Item: WithLayout>,
{
    let mut offsets = buffer_offsets(fields, capacity);
    offsets.by_ref().try_for_each(|offset| offset.map(drop))?;
    Ok(offsets.into_buffer())
}
