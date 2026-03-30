use core::alloc::{Layout, LayoutError};

pub use self::{
    buffer_offsets::{BufferOffset, BufferOffsets, RawBufferOffsets},
    copied_descriptors::{CopiedFieldDescriptors, IntoCopiedFieldDescriptors},
    descriptors::{
        FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput,
        FieldDescriptorsOwned,
    },
};

mod buffer_offsets;
mod copied_descriptors;
mod descriptors;

/// Calculates offsets for each provided region in a single buffer.
#[inline]
pub fn buffer_offsets<I>(fields: I, capacity: usize) -> BufferOffsets<I::IntoIter>
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    let state = RawBufferOffsets::new(capacity);
    let fields = fields.into_iter();
    unsafe { BufferOffsets::from_parts(state, fields) }
}

/// Calculates layout needed to store provided regions in a single buffer.
#[inline]
pub fn buffer_layout<I>(fields: I, capacity: usize) -> Result<Layout, LayoutError>
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    let mut offsets = buffer_offsets(fields, capacity);
    offsets.by_ref().try_for_each(|offset| offset.map(drop))?;
    Ok(offsets.into_layout())
}
