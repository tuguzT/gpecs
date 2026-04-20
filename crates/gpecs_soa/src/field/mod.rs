use core::alloc::{Layout, LayoutError};

use crate::layout::WithLayout;

pub use self::{
    buffer_offsets::{BufferOffset, BufferOffsets, RawBufferOffsets},
    field_layouts::{
        FieldLayouts, FieldLayoutsItem, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned,
    },
    into_layouts::{IntoFieldLayouts, IntoFieldLayoutsIter},
};

mod buffer_offsets;
mod field_layouts;
mod into_layouts;

/// Calculates offsets for each provided region in a single buffer.
#[inline]
pub fn buffer_offsets<I>(fields: I, capacity: usize) -> BufferOffsets<I::IntoIter>
where
    I: IntoIterator<Item: WithLayout>,
{
    let state = RawBufferOffsets::new(capacity);
    let fields = fields.into_iter();
    unsafe { BufferOffsets::from_parts(state, fields) }
}

/// Calculates layout needed to store provided regions in a single buffer.
#[inline]
pub fn buffer_layout<I>(fields: I, capacity: usize) -> Result<Layout, LayoutError>
where
    I: IntoIterator<Item: WithLayout>,
{
    let mut offsets = buffer_offsets(fields, capacity);
    offsets.by_ref().try_for_each(|offset| offset.map(drop))?;
    Ok(offsets.into_buffer_layout())
}
